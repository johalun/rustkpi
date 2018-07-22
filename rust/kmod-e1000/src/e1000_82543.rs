
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
    // 	struct e1000_dev_spec_82543 *dev_spec = &hw->dev_spec._82543;
    // 	DEBUGFUNC("e1000_set_tbi_compatibility_82543");
    // 	if (hw->mac.type != e1000_82543) {
    // 		DEBUGOUT("TBI compatibility workaround for 82543 only.\n");
    // 		goto out;
    // 	}
    // 	if (state)
    // 		dev_spec->tbi_compatibility |= TBI_COMPAT_ENABLED;
    // 	else
    // 		dev_spec->tbi_compatibility &= ~TBI_COMPAT_ENABLED;
    // out:
    // 	return;

    if adapter.hw.mac.mac_type != MacType::Mac_82543 {
        e1000_println!("Only for 82543 - do nothing and return");
        return;
    }
}
