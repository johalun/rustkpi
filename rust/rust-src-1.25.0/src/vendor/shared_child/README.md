# shared_child.rs [![Travis build](https://travis-ci.org/oconnor663/shared_child.rs.svg?branch=master)](https://travis-ci.org/oconnor663/shared_child.rs) [![Build status](https://ci.appveyor.com/api/projects/status/900ckow3c5awq3t5/branch/master?svg=true)](https://ci.appveyor.com/project/oconnor663/shared-child-rs/branch/master) [![crates.io](https://img.shields.io/crates/v/shared_child.svg)](https://crates.io/crates/shared_child) [![docs.rs](https://docs.rs/shared_child/badge.svg)](https://docs.rs/shared_child)

A library for awaiting and killing child processes from multiple threads.

The
[`std::process::Child`](https://doc.rust-lang.org/std/process/struct.Child.html)
type in the standard library provides
[`wait`](https://doc.rust-lang.org/std/process/struct.Child.html#method.wait)
and
[`kill`](https://doc.rust-lang.org/std/process/struct.Child.html#method.kill)
methods that take `&mut self`, making it impossible to kill a child process
while another thread is waiting on it. That design works around a race
condition in Unix's `waitpid` function, where a PID might get reused as soon
as the wait returns, so a signal sent around the same time could
accidentally get delivered to the wrong process.

However with the newer POSIX `waitid` function, we can wait on a child
without freeing its PID for reuse. That makes it safe to send signals
concurrently. Windows has actually always supported this, by preventing PID
reuse while there are still open handles to a child process. This library
wraps `std::process::Child` for concurrent use, backed by these APIs.

- [Docs](https://docs.rs/shared_child)
- [Crate](https://crates.io/crates/shared_child)
- [Repo](https://github.com/oconnor663/shared_child.rs)
