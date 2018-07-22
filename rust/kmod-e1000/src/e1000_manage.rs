
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

    // u32 manc;
    // u32 fwsm, factps;

    // DEBUGFUNC("e1000_enable_mng_pass_thru");

    // if (!hw->mac.asf_firmware_present)
    // 	return FALSE;

    if !adapter.hw.mac.asf_firmware_present {
        return false;
    }

    // manc = E1000_READ_REG(hw, E1000_MANC);
    let manc: u32 = do_read_register(adapter, E1000_MANC);

    // if (!(manc & E1000_MANC_RCV_TCO_EN))
    // 	return FALSE;
    if manc & E1000_MANC_RCV_TCO_EN == 0 {
        return false;
    }

    // if (hw->mac.has_fwsm) {
    // 	fwsm = E1000_READ_REG(hw, E1000_FWSM);
    // 	factps = E1000_READ_REG(hw, E1000_FACTPS);
    // 	if (!(factps & E1000_FACTPS_MNGCG) &&
    // 	    ((fwsm & E1000_FWSM_MODE_MASK) ==
    // 	     (e1000_mng_mode_pt << E1000_FWSM_MODE_SHIFT)))
    // 		return TRUE;
    // } else if ((hw->mac.type == e1000_82574) ||
    // 	   (hw->mac.type == e1000_82583)) {
    // 	u16 data;
    // 	s32 ret_val;
    // 	factps = E1000_READ_REG(hw, E1000_FACTPS);
    // 	ret_val = e1000_read_nvm(hw, NVM_INIT_CONTROL2_REG, 1, &data);
    // 	if (ret_val)
    // 		return FALSE;

    // 	if (!(factps & E1000_FACTPS_MNGCG) &&
    // 	    ((data & E1000_NVM_INIT_CTRL2_MNGM) ==
    // 	     (e1000_mng_mode_pt << 13)))
    // 		return TRUE;
    // } else if ((manc & E1000_MANC_SMBUS_EN) &&
    // 	   !(manc & E1000_MANC_ASF_EN)) {
    // 	return TRUE;
    // }
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

    // return FALSE;
    false
}
