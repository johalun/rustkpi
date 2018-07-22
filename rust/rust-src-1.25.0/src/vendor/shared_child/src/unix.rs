extern crate libc;

use std;
use std::io;
use std::process::{Child, ExitStatus};

// A handle on Unix is just the PID.
pub struct Handle(u32);

pub fn get_handle(child: &Child) -> Handle {
    Handle(child.id())
}

// This blocks until a child exits, without reaping the child.
pub fn wait_without_reaping(handle: &Handle) -> io::Result<()> {
    loop {
        let ret = unsafe {
            let mut siginfo = std::mem::uninitialized();
            libc::waitid(libc::P_PID,
                         handle.0 as libc::id_t,
                         &mut siginfo,
                         libc::WEXITED | libc::WNOWAIT)
        };
        if ret == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
        // We were interrupted. Loop and retry.
    }
}

// This reaps the child if it's already exited, but doesn't block otherwise.
// There's an unstable Child::try_wait() function in libstd right now, and when
// that stabilizes we can probably delete this.
pub fn try_wait(handle: &Handle) -> io::Result<Option<ExitStatus>> {
    let mut status = 0;
    let waitpid_ret = unsafe { libc::waitpid(handle.0 as libc::pid_t, &mut status, libc::WNOHANG) };
    if waitpid_ret < 0 {
        // EINTR is not possible with WNOHANG, so no need to retry.
        Err(io::Error::last_os_error())
    } else if waitpid_ret == 0 {
        Ok(None)
    } else {
        use std::os::unix::process::ExitStatusExt;
        Ok(Some(ExitStatus::from_raw(status)))
    }
}
