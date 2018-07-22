#![allow(dead_code, improper_ctypes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_imports)]
#![feature(const_fn)]

mod sys;

#[allow(unused_macros)]
#[macro_use]
mod macros;

pub use sys::raw;

pub use sys::callout_sys;
pub use sys::conf_sys;
pub use sys::iflib_sys;
pub use sys::kernel_sys;
pub use sys::kthread_sys;
pub use sys::malloc_sys;
pub use sys::module_sys;
pub use sys::param_sys;
pub use sys::systm_sys;
pub use sys::types_sys;
pub use sys::uio_sys;
pub use sys::unistd_sys;


unsafe impl Sync for module_sys::moduledata_t {}
unsafe impl Sync for module_sys::mod_metadata {}
unsafe impl Sync for module_sys::mod_version {}
unsafe impl Sync for kernel_sys::sysinit {}
