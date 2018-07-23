#![no_std]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(macro_reexport)]
#![feature(core_intrinsics)]
#![feature(prelude_import)]
#![feature(raw)]
#![feature(slice_concat_ext)]
#![feature(unicode)]

// To use custom allocator
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(alloc_system)]
#![feature(global_allocator)]
#![default_lib_allocator]
#![feature(allocator_internals)]
#[global_allocator]
static ALLOC: alloc_kernel::System = alloc_kernel::System;

// We want to reexport a few macros from core but libcore has already been
// imported by the compiler (via our #[no_std] attribute) In this case we just
// add a new crate name so we can attach the reexports to it.
#[macro_reexport(assert, assert_eq, assert_ne, debug_assert, debug_assert_eq, debug_assert_ne,
                 unreachable, unimplemented, write, writeln, try)]
extern crate core as __core;

#[macro_use]
#[macro_reexport(vec, format)]
extern crate alloc;

extern crate alloc_kernel;
extern crate std_unicode;
extern crate spin;

#[macro_use]
mod macros;

// Explicitly import the prelude. The compiler uses this same unstable attribute
// to import the prelude implicitly when building crates that depend on std.
#[prelude_import]
#[allow(unused)]
use prelude::v1::*;

// The Rust prelude
pub mod prelude;

#[allow(dead_code, improper_ctypes, non_camel_case_types, non_snake_case, non_upper_case_globals)]
pub mod sys;

pub mod io;
pub mod error;

// Re-export modules from libcore
pub use core::any;
pub use core::cell;
pub use core::clone;
pub use core::cmp;
pub use core::convert;
pub use core::default;
pub use core::hash;
pub use core::iter;
pub use core::intrinsics;
pub use core::marker;
pub use core::mem;
pub use core::ops;
pub use core::ptr;
pub use core::raw;
pub use core::result;
pub use core::option;
pub use core::isize;
pub use core::i8;
pub use core::i16;
pub use core::i32;
pub use core::i64;
// pub use core::i128;
pub use core::usize;
pub use core::u8;
pub use core::u16;
pub use core::u32;
pub use core::u64;
// pub use core::u128;
pub use alloc::boxed;
pub use alloc::rc;
pub use alloc::borrow;
pub use alloc::fmt;
pub use alloc::slice;
pub use alloc::str;
pub use alloc::string;
pub use alloc::vec;
pub use std_unicode::char;

pub mod sync {
    pub use alloc::arc::Arc;
    pub use alloc::arc::Weak;
    pub use spin::Mutex;
    pub use spin::MutexGuard;

}

// Copy std structure
mod kernel {
    pub use clone;
    pub use default;
    pub use error;
    pub use fmt;
    pub use io;
    pub use mem;
    pub use option;
    pub use sys;
}

// Soft float functions that are missing.
// We don't use floats in the kernel anyway so just keep
// empty impl for now.
// TODO: Patch core to remove float completely?
#[no_mangle]
pub extern "C" fn __eqsf2() {}
#[no_mangle]
pub extern "C" fn __eqdf2() {}
#[no_mangle]
pub extern "C" fn __floatundisf() {}
#[no_mangle]
pub extern "C" fn __floatundidf() {}
#[no_mangle]
pub extern "C" fn __mulsf3() {}
#[no_mangle]
pub extern "C" fn __muldf3() {}
#[no_mangle]
pub extern "C" fn __divsf3() {}
#[no_mangle]
pub extern "C" fn __divdf3() {}

// 128 bit integer stuff (we don't use it so stub ok for now...)
#[no_mangle]
pub extern "C" fn __umodti3() {}
#[no_mangle]
pub extern "C" fn __muloti4() {}
#[no_mangle]
pub extern "C" fn __udivti3() {}


#[no_mangle]
pub extern "C" fn puts() {}


#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

// This function may be needed based on the compilation target.
#[lang = "eh_unwind_resume"]
#[no_mangle]
pub extern "C" fn rust_eh_unwind_resume() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn rust_begin_panic(
    msg: core::fmt::Arguments,
    file: &'static str,
    line: u32,
    column: u32,
) -> ! {
    unsafe {
        sys::systm_sys::log(
            0,
            "<<<<<<<<<<<<<<<<<<<<<<<< RUST PANIC >>>>>>>>>>>>>>>>>>>>>>>>\n\0".as_ptr() as *const i8,
        );
        let m = &format!("{}:{}:{}\n{}\n\0", file, line, column, msg);
        kernel::sys::systm_sys::log(0, m.as_ptr() as *const i8);
    }
    loop {}
}
