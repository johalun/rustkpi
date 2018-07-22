//
//

#![no_std]

// #![deny(missing_docs)]
// #![deny(missing_debug_implementations)]

//
// Tell the compiler to link to either panic_abort or panic_unwind
// #![needs_panic_runtime]

// Turn warnings into errors, but only after stage0, where it can be useful for
// code to emit warnings during language transitions
// #![deny(warnings)]

// std may use features in a platform-specific way
// #![allow(unused_features,unused_variables, unused_mut, dead_code)]


// Unknown features
//
// #![feature(global_asm)]
// #![feature(used)]
// #![feature(allow_internal_unsafe)]
// #![feature(allow_internal_unstable)]
// #![feature(align_offset)]
// #![feature(asm)]
// #![feature(box_syntax)]
// #![feature(cfg_target_has_atomic)]
// #![feature(cfg_target_thread_local)]
// #![feature(cfg_target_vendor)]
// #![feature(char_error_internals)]
// #![feature(char_internals)]
// #![feature(collections_range)]
// #![feature(compiler_builtins_lib)]
#![feature(const_fn)]
// #![feature(core_float)]
// #![feature(dropck_eyepatch)]
// #![feature(exact_size_is_empty)]
// #![feature(float_from_str_radix)]
// #![feature(fn_traits)]
// #![feature(fnbox)]
// #![feature(fused)]
// #![feature(generic_param_attrs)]
// #![feature(hashmap_hasher)]
// #![feature(heap_api)]
// #![feature(i128)]
// #![feature(i128_type)]
// #![feature(inclusive_range)]
// #![feature(int_error_internals)]
// #![feature(integer_atomics)]
// #![feature(into_cow)]
// #![feature(libc)]
// #![feature(link_args)]
// #![feature(linkage)]
// #![feature(macro_vis_matcher)]
// #![feature(needs_panic_runtime)]
// #![feature(never_type)]
// #![feature(num_bits_bytes)]
// #![feature(old_wrapping)]
// #![feature(on_unimplemented)]
// #![feature(oom)]
// #![feature(optin_builtin_traits)]
// #![feature(panic_unwind)]
// #![feature(peek)]
// #![feature(placement_in_syntax)]
// #![feature(placement_new_protocol)]
// #![feature(rand)]
// #![feature(repr_simd)]
// #![feature(rustc_attrs)]
// #![cfg_attr(not(stage0), feature(rustc_const_unstable))]
// #![feature(shared)]
// #![feature(sip_hash_13)]
// #![feature(slice_bytes)]
// #![feature(slice_patterns)]
// #![feature(staged_api)]
// #![feature(stmt_expr_attributes)]
// #![feature(str_char)]
// #![feature(str_internals)]
// #![feature(str_utf16)]
// #![feature(test, rustc_private)]
// #![feature(thread_local)]
// #![feature(toowned_clone_into)]
// #![feature(try_from)]
// #![feature(unboxed_closures)]
// #![feature(unique)]
// #![feature(untagged_unions)]
// #![feature(unwind_attributes)]
// #![feature(vec_push_all)]
// #![feature(doc_cfg)]
// #![feature(doc_masked)]
// #![cfg_attr(test, feature(update_panic_count))]
// #![cfg_attr(not(stage0), feature(const_max_value))]
// #![cfg_attr(not(stage0), feature(const_atomic_bool_new))]
// #![cfg_attr(not(stage0), feature(const_atomic_isize_new))]
// #![cfg_attr(not(stage0), feature(const_atomic_usize_new))]
// #![cfg_attr(all(not(stage0), windows), feature(const_atomic_ptr_new))]
// #![cfg_attr(not(stage0), feature(const_unsafe_cell_new))]
// #![cfg_attr(not(stage0), feature(const_cell_new))]
// #![cfg_attr(not(stage0), feature(const_once_new))]
// #![cfg_attr(not(stage0), feature(const_ptr_null))]
// #![cfg_attr(not(stage0), feature(const_ptr_null_mut))]



#![feature(lang_items)]
#![feature(macro_reexport)]

// To use core::intrinsics (atomic_* functions etc)
#![feature(core_intrinsics)]

#![feature(prelude_import)]

// To use core::raw
#![feature(raw)]

// Imported in prelude
#![feature(slice_concat_ext)]

// To use std_unicode, required by alloc
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

// extern crate compiler_builtins;
// extern crate kernel_sys;

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


// module_version!(rustkpi, 1);
// module_depend!(rustkpi, pci, 1, 1, 1);
