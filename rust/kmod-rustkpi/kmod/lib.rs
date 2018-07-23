#![feature(plugin, used, global_asm, rustc_private)]
#![feature(trace_macros)]
#![plugin(interpolate_idents)]
#![allow(unused_imports, non_upper_case_globals)]
#![no_std]
#![feature(const_fn)]

#[macro_use]
extern crate kernel;
use kernel::prelude::v1::*;
use kernel::sys::raw::*;
use kernel::sys::*;
use kernel::sys::sysctl_sys::*;
use kernel::sys;
use core::ptr;

pub extern "C" fn module_event(_module: sys::module_t, event: c_int, _arg: *mut c_void) -> c_int {

    match ModEventType::from(event) {
        ModEventType::Load => {
            println!("==> RustKPI loaded");
        }
        ModEventType::Unload => {
            println!("==> RustKPI unloaded");
        }
        ModEventType::Quiesce => {}
        ModEventType::Shutdown => {}
        ModEventType::Unknown => {}
    }
    0
}

pub static MODULE_DATA: sys::moduledata_t = sys::moduledata_t {
    name: b"rustkpi\0" as *const u8 as *const i8,
    evhand: Some(module_event),
    priv_: 0 as *mut c_void,
};

// These macros require interpolate_idents
declare_module!(
    rustkpi,
    MODULE_DATA,
    sys::sysinit_sub_id::SI_SUB_DRIVERS,
    sys::sysinit_elem_order::SI_ORDER_MIDDLE
);


// These macros require interpolate_idents
module_version!(rustkpi, 1);
module_depend!(rustkpi, pci, 1, 1, 1);
