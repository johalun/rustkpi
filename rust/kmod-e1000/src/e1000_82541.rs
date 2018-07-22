
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


pub fn init_script_state(adapter: &mut Adapter, state: bool) {
    e1000_println!();
    // 	struct e1000_dev_spec_82541 *dev_spec = &hw->dev_spec._82541;
    // 	DEBUGFUNC("e1000_init_script_state_82541");
    // 	if (hw->phy.type != e1000_phy_igp) {
    // 		DEBUGOUT("Initialization script not necessary.\n");
    // 		goto out;
    // 	}
    // 	dev_spec->phy_init_script = state;
    // out:
    // 	return;

    if adapter.hw.phy.phy_type != PhyType::Type_igp {
        e1000_println!("Initialization script not necessary");
        return;
    }
    unsafe {
        adapter.hw.dev_spec._82541.phy_init_script = state;
    }
}
