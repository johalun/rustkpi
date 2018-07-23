
use kernel;
use kernel::ptr::Unique;

use kernel::sys::raw::*;
use kernel::prelude::v1::*;

use sys::e1000::*;

use adapter::*;
use iflib::*;
use hw::*;
use consts::*;
use bridge::*;
use e1000_regs;
use e1000_osdep::*;

pub fn translate_register(reg: u32) -> u32 {
    e1000_println!();
    // Only does something for 82543.
    incomplete!();
    reg
}
