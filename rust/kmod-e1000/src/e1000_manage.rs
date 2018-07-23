
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
use e1000_osdep::*;



pub fn enable_mng_pass_thru(adapter: &mut Adapter) -> bool {
    e1000_println!();

    if !adapter.hw.mac.asf_firmware_present {
        return false;
    }

    let manc: u32 = do_read_register(adapter, E1000_MANC);

    if manc & E1000_MANC_RCV_TCO_EN == 0 {
        return false;
    }

    if adapter.hw.mac.has_fwsm {
        let fwsm = do_read_register(adapter, E1000_FWSM);
        let factps = do_read_register(adapter, E1000_FACTPS);
        if (factps & E1000_FACTPS_MNGCG == 0) &&
            ((fwsm & E1000_FWSM_MODE_MASK) == ((MngMode::Pt as u32) << E1000_FWSM_MODE_SHIFT))
        {
            return true;
        }
    } else if adapter.hw.mac.mac_type == MacType::Mac_82574 ||
               adapter.hw.mac.mac_type == MacType::Mac_82583
    {
        incomplete!();
    } else if manc & E1000_MANC_SMBUS_EN > 0 && manc & E1000_MANC_ASF_EN == 0 {
        return true;
    }

    false
}
