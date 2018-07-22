#![feature(const_fn,plugin,used,global_asm,rustc_private)]
#![plugin(interpolate_idents)]
#![no_std]

#[macro_use]
extern crate kernel;

use kernel::sys::ModEventType;
use kernel::sys::module_sys::module_t;
use kernel::sys::module_sys::moduledata_t;
use kernel::sys::kernel_sys::sysinit_sub_id;
use kernel::sys::kernel_sys::sysinit_elem_order;
use kernel::sys::raw::c_int;
use kernel::sys::raw::c_void;

#[derive(Debug)]
struct A(i32);
impl kernel::ops::Drop for A {
    fn drop(&mut self) {
        println!("rustkpi-hello: A::drop() {:?}", self);
    }
}
/*
 * The kernel expects a C function for the event callback.
 */
pub extern "C" fn module_event(_module: module_t, event: c_int, _arg: *mut c_void) -> c_int {
    match ModEventType::from(event) {
        ModEventType::Load => {
            println!("rustkpi-hello: Got kernel module event: LOAD");
            /*
             * Create a vector with heap allocated storage.
             * Control that it is released when the
             * variable 'v' goes out of scope by watching
             * the output from A's drop() function.
             */
            let mut v = vec![A(0), A(1), A(2)];
            println!("rustkpi-hello: Vector is: {:?}", v);
        }
        ModEventType::Unload => {
            println!("rustkpi-hello: Got kernel module event: UNLOAD");
        }
        ModEventType::Quiesce => {}
        ModEventType::Shutdown => {}
        ModEventType::Unknown => {}
    }
    0
}

pub static MODULE_DATA: moduledata_t = moduledata_t {
    name: b"rustkpi_hello\0" as *const u8 as *const i8,
    evhand: Some(module_event),
    priv_: 0 as *mut c_void,
};

/* These macros require interpolate_idents compiler plugin */
declare_module!(
    rustkpi_hello,
    MODULE_DATA,
    sysinit_sub_id::SI_SUB_DRIVERS,
    sysinit_elem_order::SI_ORDER_MIDDLE
);
module_depend!(rustkpi_hello, rustkpi, 1, 1, 1);
