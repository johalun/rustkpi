
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


pub fn set_tbi_compatibility(adapter: &mut Adapter, state: bool) {
    e1000_println!();
    if adapter.hw.mac.mac_type != MacType::Mac_82543 {
        return;
    }
    // Only for 82543 which is unsupported so do nothing
}
