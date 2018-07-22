Thread-ID
=========
Get a unique ID for the current thread in Rust.

[![Build Status][tr-img]][tr]
[![Build Status][av-img]][av]
[![Crates.io version][crate-img]][crate]
[![Documentation][docs-img]][docs]

For diagnostics and debugging it can often be useful to get an ID that is
different for every thread. The standard library does not expose a way to do
that, hence this crate.

Example
-------

```rust
use std::thread;
use thread_id;

thread::spawn(move || {
    println!("spawned thread has id {}", thread_id::get());
});

println!("main thread has id {}", thread_id::get());
```

This will print two different numbers.

License
-------
Thread-ID is licensed under the [Apache 2.0][apache2] license. It may be used
in free software as well as closed-source applications, both for commercial and
non-commercial use under the conditions given in the license. If you want to use
Thread-ID in your GPLv2-licensed software, you can add an [exception][except]
to your copyright notice.

[tr-img]:    https://travis-ci.org/ruud-v-a/thread-id.svg?branch=master
[tr]:        https://travis-ci.org/ruud-v-a/thread-id
[av-img]:    https://ci.appveyor.com/api/projects/status/a6ccbm3x4fgi6wku?svg=true
[av]:        https://ci.appveyor.com/project/ruud-v-a/thread-id
[crate-img]: http://img.shields.io/crates/v/thread-id.svg
[crate]:     https://crates.io/crates/thread-id
[docs-img]:  http://img.shields.io/badge/docs-online-blue.svg
[docs]:      https://ruud-v-a.github.io/thread-id/doc/v2.0.0/thread-id/
[apache2]:   https://www.apache.org/licenses/LICENSE-2.0
[except]:    https://www.gnu.org/licenses/gpl-faq.html#GPLIncompatibleLibs
