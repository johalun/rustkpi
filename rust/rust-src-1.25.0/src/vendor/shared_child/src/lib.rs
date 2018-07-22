//! A library for awaiting and killing child processes from multiple threads.
//!
//! The
//! [`std::process::Child`](https://doc.rust-lang.org/std/process/struct.Child.html)
//! type in the standard library provides
//! [`wait`](https://doc.rust-lang.org/std/process/struct.Child.html#method.wait)
//! and
//! [`kill`](https://doc.rust-lang.org/std/process/struct.Child.html#method.kill)
//! methods that take `&mut self`, making it impossible to kill a child process
//! while another thread is waiting on it. That design works around a race
//! condition in Unix's `waitpid` function, where a PID might get reused as soon
//! as the wait returns, so a signal sent around the same time could
//! accidentally get delivered to the wrong process.
//!
//! However with the newer POSIX `waitid` function, we can wait on a child
//! without freeing its PID for reuse. That makes it safe to send signals
//! concurrently. Windows has actually always supported this, by preventing PID
//! reuse while there are still open handles to a child process. This library
//! wraps `std::process::Child` for concurrent use, backed by these APIs.
//!
//! - [Docs](https://docs.rs/shared_child)
//! - [Crate](https://crates.io/crates/shared_child)
//! - [Repo](https://github.com/oconnor663/shared_child.rs)
//!
//! Compatibility note: The `libc` crate doesn't currently support `waitid` on
//! NetBSD or OpenBSD, or on older versions of OSX. There [might also
//! be](https://bugs.python.org/msg167016) some version of OSX where the
//! `waitid` function exists but is broken. We can add a "best effort"
//! workaround using `waitpid` for these platforms as we run into them. Please
//! [file an issue](https://github.com/oconnor663/shared_child.rs/issues/new) if
//! you hit this.
//!
//! # Example
//!
//! ```rust
//! # fn run() -> std::io::Result<()> {
//! use shared_child::SharedChild;
//! use std::process::Command;
//! use std::sync::Arc;
//!
//! // Spawn a child that will just sleep for a long time,
//! // and put it in an Arc to share between threads.
//! let mut command = Command::new("python");
//! command.arg("-c").arg("import time; time.sleep(1000000000)");
//! let shared_child = SharedChild::spawn(&mut command)?;
//! let child_arc = Arc::new(shared_child);
//!
//! // On another thread, wait on the child process.
//! let child_arc_clone = child_arc.clone();
//! let thread = std::thread::spawn(move || {
//!     child_arc_clone.wait()
//! });
//!
//! // While the other thread is waiting, kill the child process.
//! // This wouldn't be possible with e.g. Arc<Mutex<Child>> from
//! // the standard library, because the waiting thread would be
//! // holding the mutex.
//! child_arc.kill()?;
//!
//! // Join the waiting thread and get the exit status.
//! let exit_status = thread.join().unwrap()?;
//! assert!(!exit_status.success());
//! # Ok(())
//! # }
//! # fn main() { run().unwrap(); }
//! ```

use std::io;
use std::process::{Command, Child, ExitStatus};
use std::sync::{Condvar, Mutex};

#[cfg(not(windows))]
#[path="unix.rs"]
mod sys;
#[cfg(windows)]
#[path="windows.rs"]
mod sys;

pub struct SharedChild {
    // This lock provides shared access to kill() and wait(), though sometimes
    // we use libc::waitpid() to reap the child instead.
    child: Mutex<Child>,
    id: u32,
    handle: sys::Handle,

    // When there are multiple waiting threads, one of them will actually wait
    // on the child, and the rest will block on this condvar.
    state_lock: Mutex<ChildState>,
    state_condvar: Condvar,
}

impl SharedChild {
    /// Spawn a new `SharedChild` from a `std::process::Command`.
    pub fn spawn(command: &mut Command) -> io::Result<SharedChild> {
        let child = command.spawn()?;
        Ok(SharedChild {
            id: child.id(),
            handle: sys::get_handle(&child),
            child: Mutex::new(child),
            state_lock: Mutex::new(NotWaiting),
            state_condvar: Condvar::new(),
        })
    }

    /// Return the child process ID.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Wait for the child to exit, blocking the current thread, and return its
    /// exit status.
    pub fn wait(&self) -> io::Result<ExitStatus> {
        let mut state = self.state_lock.lock().unwrap();
        loop {
            match *state {
                NotWaiting => {
                    // Either no one is waiting on the child yet, or a previous
                    // waiter failed. That means we need to do it ourselves.
                    // Break out of this loop.
                    break;
                }
                Waiting => {
                    // Another thread is already waiting on the child. We'll
                    // block until it signal us on the condvar, then loop again.
                    // Spurious wakeups could bring us here multiple times
                    // though, see the Condvar docs.
                    state = self.state_condvar.wait(state).unwrap();
                }
                Exited(exit_status) => return Ok(exit_status),
            }
        }

        // If we get here, we have the state lock, and we're the thread
        // responsible for waiting on the child. Set the state to Waiting and
        // then release the state lock, so that other threads can observe it
        // while we block. Afterwards we must leave the Waiting state before
        // this function exits, or other waiters will deadlock.
        *state = Waiting;
        drop(state);

        // Block until the child exits without reaping it. (On Unix, that means
        // we need to call libc::waitid with the WNOWAIT flag. On Windows
        // waiting never reaps.) That makes it safe for another thread to kill
        // while we're here, without racing against some process reusing the
        // child's PID. Having only one thread in this section is important,
        // because POSIX doesn't guarantee much about what happens when multiple
        // threads wait on a child at the same time:
        // http://pubs.opengroup.org/onlinepubs/9699919799/functions/V2_chap02.html#tag_15_13
        let noreap_result = sys::wait_without_reaping(&self.handle);

        // Now either we hit an error, or the child has exited and needs to be
        // reaped. Retake the state lock and handle all the different exit
        // cases. No matter what happened/happens, we'll leave the Waiting state
        // and signal the state condvar.
        let mut state = self.state_lock.lock().unwrap();
        let final_result = noreap_result.and_then(|_| {
            // Reap the child. Errors only short-circuit this closure.
            if let Some(exit_status) = sys::try_wait(&self.handle)? {
                Ok(exit_status)
            } else {
                // This should never happen, unless waitid lied to us.
                Err(io::Error::new(io::ErrorKind::Other, "blocking wait after child exit"))
            }
        });
        *state = if let Ok(exit_status) = final_result {
            Exited(exit_status)
        } else {
            NotWaiting
        };
        self.state_condvar.notify_all();
        final_result
    }

    /// Return the child's exit status if it has already exited. If the child is
    /// still running, return `Ok(None)`.
    pub fn try_wait(&self) -> io::Result<Option<ExitStatus>> {
        let mut status = self.state_lock.lock().unwrap();

        // Unlike wait() above, we don't loop on the Condvar here. If the status
        // is Waiting or Exited, we return immediately. However, if the status
        // is NotWaiting, we'll do a non-blocking wait below, in case the child
        // has already exited.
        match *status {
            NotWaiting => {}
            Waiting => return Ok(None),
            Exited(exit_status) => return Ok(Some(exit_status)),
        };

        // No one is waiting on the child. Check to see if it's already exited.
        // If it has, put ourselves in the Exited state. (There can't be any
        // other waiters to signal, because the state was NotWaiting when we
        // started, and we're still holding the status lock.)
        if let Some(exit_status) = sys::try_wait(&self.handle)? {
            *status = Exited(exit_status);
            Ok(Some(exit_status))
        } else {
            Ok(None)
        }
    }

    /// Send a kill signal to the child. You should call `wait` afterwards to
    /// avoid leaving a zombie on Unix.
    pub fn kill(&self) -> io::Result<()> {
        let status = self.state_lock.lock().unwrap();
        if let Exited(_) = *status {
            return Ok(());
        }
        // The child is still running. Kill it.
        self.child.lock().unwrap().kill()
    }
}

enum ChildState {
    NotWaiting,
    Waiting,
    Exited(ExitStatus),
}

use ChildState::*;

#[cfg(test)]
mod tests {
    use std;
    use std::process::Command;
    use std::sync::Arc;
    use super::{SharedChild, sys};

    fn true_cmd() -> Command {
        let mut cmd = Command::new("python");
        cmd.arg("-c").arg("");
        cmd
    }

    fn sleep_forever_cmd() -> Command {
        let mut cmd = Command::new("python");
        cmd.arg("-c").arg("import time; time.sleep(1000000000)");
        cmd
    }

    #[test]
    fn test_wait() {
        let child = SharedChild::spawn(&mut true_cmd()).unwrap();
        // Test the id() function while we're at it.
        assert!(child.id() > 0);
        let status = child.wait().unwrap();
        assert_eq!(status.code().unwrap(), 0);
    }

    #[test]
    fn test_kill() {
        let child = SharedChild::spawn(&mut sleep_forever_cmd()).unwrap();
        child.kill().unwrap();
        let status = child.wait().unwrap();
        assert!(!status.success());
    }

    #[test]
    fn test_try_wait() {
        let child = SharedChild::spawn(&mut sleep_forever_cmd()).unwrap();
        let maybe_status = child.try_wait().unwrap();
        assert_eq!(maybe_status, None);
        child.kill().unwrap();
        // The child will handle that signal asynchronously, so we check it
        // repeatedly in a busy loop.
        let mut maybe_status = None;
        while let None = maybe_status {
            maybe_status = child.try_wait().unwrap();
        }
        assert!(maybe_status.is_some());
        assert!(!maybe_status.unwrap().success());
    }

    #[test]
    fn test_many_waiters() {
        let child = Arc::new(SharedChild::spawn(&mut sleep_forever_cmd()).unwrap());
        let mut threads = Vec::new();
        for _ in 0..10 {
            let clone = child.clone();
            threads.push(std::thread::spawn(move || clone.wait()));
        }
        child.kill().unwrap();
        for thread in threads {
            thread.join().unwrap().unwrap();
        }
    }

    #[test]
    fn test_waitid_after_exit_doesnt_hang() {
        // There are ominous reports (https://bugs.python.org/issue10812) of a
        // broken waitid implementation on OSX, which might hang forever if it
        // tries to wait on a child that's already exited.
        let child = true_cmd().spawn().unwrap();
        let handle = sys::get_handle(&child);
        sys::wait_without_reaping(&handle).unwrap();
        // At this point the child has definitely exited. Wait again to test
        // that a second wait doesn't block.
        sys::wait_without_reaping(&handle).unwrap();
    }
}
