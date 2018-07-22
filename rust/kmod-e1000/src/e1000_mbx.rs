
use kernel;
use kernel::ptr::Unique;

use kernel::sys::raw::*;
use kernel::prelude::v1::*;

use sys::e1000::*;

use iflib::*;
use hw::*;
use consts::*;
use bridge::*;
use adapter::*;
use e1000_osdep::*;

pub fn read_posted(arg1: &mut Adapter, arg2: &mut u32, arg3: u16, arg4: u16) -> AdResult {
    e1000_println!();
    incomplete!();
    Ok(())

}

pub fn write_posted(arg1: &mut Adapter, arg2: &mut u32, arg3: u16, arg4: u16) -> AdResult {
    e1000_println!();
    incomplete!();
    Ok(())
}
