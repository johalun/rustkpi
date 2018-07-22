
use kernel;
use kernel::ptr::Unique;

use kernel::sys::raw::*;
use kernel::prelude::v1::*;

use sys::e1000::*;
use sys::e1000_consts::*;

use iflib::*;
use hw::*;
use consts::*;
use bridge::*;
use adapter::*;
use e1000_osdep::*;
use e1000_regs::*;
use e1000_phy;

use kernel::sys::iflib_sys::ETHER_ADDR_LEN;

use DEBUG_PRINT;

pub fn get_bus_info_pci_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();
    // 	struct e1000_mac_info *mac = &hw->mac;
    // 	struct e1000_bus_info *bus = &hw->bus;
    // 	u32 status = E1000_READ_REG(hw, E1000_STATUS);
    // 	s32 ret_val = E1000_SUCCESS;

    let status = do_read_register(adapter, E1000_STATUS);

    // 	DEBUGFUNC("e1000_get_bus_info_pci_generic");

    /* PCI or PCI-X? */
    // 	bus->type = (status & E1000_STATUS_PCIX_MODE)
    // 			? e1000_bus_type_pcix
    // 			: e1000_bus_type_pci;

    adapter.hw.bus.bustype = match btst!(status, E1000_STATUS_PCIX_MODE) {
        true => BusType::Pcix,
        false => BusType::Pci,
    };

    /* Bus speed */
    // 	if (bus->type == e1000_bus_type_pci) {
    // 		bus->speed = (status & E1000_STATUS_PCI66)
    // 			     ? e1000_bus_speed_66
    // 			     : e1000_bus_speed_33;
    // 	} else {
    // 		switch (status & E1000_STATUS_PCIX_SPEED) {
    // 		case E1000_STATUS_PCIX_SPEED_66:
    // 			bus->speed = e1000_bus_speed_66;
    // 			break;
    // 		case E1000_STATUS_PCIX_SPEED_100:
    // 			bus->speed = e1000_bus_speed_100;
    // 			break;
    // 		case E1000_STATUS_PCIX_SPEED_133:
    // 			bus->speed = e1000_bus_speed_133;
    // 			break;
    // 		default:
    // 			bus->speed = e1000_bus_speed_reserved;
    // 			break;
    // 		}
    // 	}

    adapter.hw.bus.speed = match adapter.hw.bus.bustype {
        BusType::Pci => {
            match status & E1000_STATUS_PCI66 {
                E1000_STATUS_PCI66 => BusSpeed::Speed_66,
                _ => BusSpeed::Speed_33,
            }
        }
        _ => {
            match status & E1000_STATUS_PCIX_SPEED {
                E1000_STATUS_PCIX_SPEED_66 => BusSpeed::Speed_66,
                E1000_STATUS_PCIX_SPEED_100 => BusSpeed::Speed_100,
                E1000_STATUS_PCIX_SPEED_133 => BusSpeed::Speed_133,
                _ => BusSpeed::Reserved,
            }
        }
    };

    /* Bus width */
    // bus->width = (status & E1000_STATUS_BUS64)
    // 	     ? e1000_bus_width_64
    // 	     : e1000_bus_width_32;

    adapter.hw.bus.width = match btst!(status, E1000_STATUS_BUS64) {
        true => BusWidth::Width_64,
        false => BusWidth::Width_32,
    };

    /* Which PCI(-X) function? */
    // 	mac->ops.set_lan_id(hw);
    if let Some(f) = adapter.hw.mac.ops.set_lan_id {
        f(adapter);
    }
    // 	return ret_val;
    Ok(())
}

/// e1000_get_bus_info_pcie_generic - Get PCIe bus information
/// @hw: pointer to the HW structure
///
/// Determines and stores the system bus information for a particular
/// network interface.  The following bus information is determined and stored:
/// bus speed, bus width, type (PCIe), and PCIe function.
pub fn get_bus_info_pcie_generic(adapter: &mut Adapter) -> AdResult {
    e1000_println!();
    // struct e1000_mac_info *mac = &hw->mac;
    // struct e1000_bus_info *bus = &hw->bus;
    // s32 ret_val;
    // u16 pcie_link_status;
    let mut pcie_link_status: u16 = 0;

    // DEBUGFUNC("e1000_get_bus_info_pcie_generic");

    // bus->type = e1000_bus_type_pci_express;
    adapter.hw.bus.bustype = BusType::Pci_express;

    // ret_val = e1000_read_pcie_cap_reg(hw, PCIE_LINK_STATUS,
    //     			      &pcie_link_status);
    // if (ret_val) {
    //     bus->width = e1000_bus_width_unknown;
    //     bus->speed = e1000_bus_speed_unknown;
    // } else {
    //     switch (pcie_link_status & PCIE_LINK_SPEED_MASK) {
    //         case PCIE_LINK_SPEED_2500:
    //         bus->speed = e1000_bus_speed_2500;
    //         break;
    //         case PCIE_LINK_SPEED_5000:
    //         bus->speed = e1000_bus_speed_5000;
    //         break;
    //         default:
    //         bus->speed = e1000_bus_speed_unknown;
    //         break;
    //     }
    //     bus->width = (enum e1000_bus_width)((pcie_link_status &
    //     		                     PCIE_LINK_WIDTH_MASK) >> PCIE_LINK_WIDTH_SHIFT);
    if let Err(e) = adapter.dev.read_pcie_cap_reg(
        PCIE_LINK_STATUS,
        &mut pcie_link_status,
    )
    {
        eprintln!("{:?}", e);
        adapter.hw.bus.width = BusWidth::Unknown;
        adapter.hw.bus.speed = BusSpeed::Unknown;
    } else {
        adapter.hw.bus.speed = match pcie_link_status & PCIE_LINK_SPEED_MASK {
            PCIE_LINK_SPEED_2500 => BusSpeed::Speed_2500,
            PCIE_LINK_SPEED_5000 => BusSpeed::Speed_5000,
            _ => BusSpeed::Unknown,
        };
        let w = (pcie_link_status & PCIE_LINK_WIDTH_MASK) >> PCIE_LINK_WIDTH_SHIFT;
        adapter.hw.bus.width = match w {
            1 => BusWidth::Width_pcie_x1,
            2 => BusWidth::Width_pcie_x2,
            4 => BusWidth::Width_pcie_x4,
            8 => BusWidth::Width_pcie_x8,
            9 => BusWidth::Width_32,
            10 => BusWidth::Width_64,
            11 => BusWidth::Reserved,
            _ => BusWidth::Unknown,
        };
    }
    // }
    // mac->ops.set_lan_id(hw);
    if let Some(f) = adapter.hw.mac.ops.set_lan_id {
        f(adapter);
    }
    // return E1000_SUCCESS;
    Ok(())
}


pub fn led_off_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();

    match adapter.hw.phy.media_type {
        MediaType::Fiber => Err("Fiber not supported yet".to_string()),
        MediaType::Copper => {
            do_write_register(adapter, E1000_LEDCTL, adapter.hw.mac.ledctl_mode1);
            Ok(())
        }
        _ => {
            eprintln!("Unknown hardware {:?}", adapter.hw.phy.media_type);
            Ok(())
        }
    }
}

pub fn led_on_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();

    match adapter.hw.phy.media_type {
        MediaType::Fiber => Err("Fiber not supported yet".to_string()),
        MediaType::Copper => {
            do_write_register(adapter, E1000_LEDCTL, adapter.hw.mac.ledctl_mode2);
            Ok(())
        }
        _ => {
            eprintln!("Unknown hardware {:?}", adapter.hw.phy.media_type);
            Ok(())
        }
    }
}

pub fn cleanup_led_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();

    do_write_register(adapter, E1000_LEDCTL, adapter.hw.mac.ledctl_default);
    Ok(())
}

pub fn setup_led_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();
    incomplete_return!();
}

pub fn id_led_init_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();
    incomplete_return!();
}

pub fn clear_vfta_generic(adapter: &mut Adapter) {
    e1000_mac_println!();

    // for (offset = 0; offset < E1000_VLAN_FILTER_TBL_SIZE; offset++) {
    //     E1000_WRITE_REG_ARRAY(hw, E1000_VFTA, offset, 0);
    //     E1000_WRITE_FLUSH(hw);
    // }

    for offset in 0..E1000_VLAN_FILTER_TBL_SIZE {
        do_write_register_array(adapter, E1000_VFTA, offset, 0);
        do_write_flush(adapter);
    }
}

pub fn write_vfta_generic(adapter: &mut Adapter, offset: u32, value: u32) {
    e1000_mac_println!();
    incomplete!();
}

pub fn hash_mc_addr_generic(adapter: &Adapter, mc_addr: &[u8]) -> u32 {
    e1000_mac_println!(
        "mc_addr: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mc_addr[0],
        mc_addr[1],
        mc_addr[2],
        mc_addr[3],
        mc_addr[4],
        mc_addr[5]
    );
    assert!(mc_addr.len() == 6);

    // u32 hash_value, hash_mask;
    // u8 bit_shift = 0;
    let mut bit_shift: u32 = 0;
    let hash_value: u32;

    // DEBUGFUNC("e1000_hash_mc_addr_generic");

    /* Register count multiplied by bits per register */
    // hash_mask = (hw->mac.mta_reg_count * 32) - 1;
    let hash_mask: u32 = (adapter.hw.mac.mta_reg_count * 32) as u32 - 1;

    /* For a mc_filter_type of 0, bit_shift is the number of left-shifts
     * where 0xFF would still fall within the hash mask.
     */
    // while (hash_mask >> bit_shift != 0xFF)
    while hash_mask >> bit_shift != 0xFF {
        // bit_shift++;
        // /* The portion of the address that is used for the hash table
        //  * is determined by the mc_filter_type setting.
        //  * The algorithm is such that there is a total of 8 bits of shifting.
        //  * The bit_shift for a mc_filter_type of 0 represents the number of
        //  * left-shifts where the MSB of mc_addr[5] would still fall within
        //  * the hash_mask.  Case 0 does this exactly.  Since there are a total
        //  * of 8 bits of shifting, then mc_addr[4] will shift right the
        //  * remaining number of bits. Thus 8 - bit_shift.  The rest of the
        //  * cases are a variation of this algorithm...essentially raising the
        //  * number of bits to shift mc_addr[5] left, while still keeping the
        //  * 8-bit shifting total.
        //  *
        //  * For example, given the following Destination MAC Address and an
        //  * mta register count of 128 (thus a 4096-bit vector and 0xFFF mask),
        //  * we can see that the bit_shift for case 0 is 4.  These are the hash
        //  * values resulting from each mc_filter_type...
        //  * [0] [1] [2] [3] [4] [5]
        //  * 01  AA  00  12  34  56
        //  * LSB		 MSB
        //  *
        //  * case 0: hash_value = ((0x34 >> 4) | (0x56 << 4)) & 0xFFF = 0x563
        //  * case 1: hash_value = ((0x34 >> 3) | (0x56 << 5)) & 0xFFF = 0xAC6
        //  * case 2: hash_value = ((0x34 >> 2) | (0x56 << 6)) & 0xFFF = 0x163
        //  * case 3: hash_value = ((0x34 >> 0) | (0x56 << 8)) & 0xFFF = 0x634
        //  */
        // switch (hw->mac.mc_filter_type) {
        //     default:
        //     case 0:
        //     break;
        //     case 1:
        //     bit_shift += 1;
        //     break;
        //     case 2:
        //     bit_shift += 2;
        //     break;
        //     case 3:
        //     bit_shift += 4;
        //     break;
        // }
        bit_shift += 1;
        match adapter.hw.mac.mc_filter_type {
            0 => (),
            1 => bit_shift += 1,
            2 => bit_shift += 2,
            3 => bit_shift += 4,
            _ => (),
        }
    }
    // hash_value = hash_mask & (((mc_addr[4] >> (8 - bit_shift)) |
    //     		       (((u16) mc_addr[5]) << bit_shift)));
    // return hash_value;

    hash_value = hash_mask &
        (((mc_addr[4] as u32) >> (8 - bit_shift)) | ((mc_addr[5] as u32) << bit_shift));
    hash_value
}

pub fn update_mc_addr_list_generic(adapter: &mut Adapter, mc_addr_count: u32) -> AdResult {
    e1000_mac_println!();

    // u32 hash_value, hash_bit, hash_reg;
    // int i;

    // DEBUGFUNC("e1000_update_mc_addr_list_generic");

    /* clear mta_shadow */
    // memset(&hw->mac.mta_shadow, 0, sizeof(hw->mac.mta_shadow));
    for mta in &mut adapter.hw.mac.mta_shadow.iter_mut() {
        *mta = 0;
    }

    /* update mta_shadow from mc_addr_list */
    // for (i = 0; (u32) i < mc_addr_count; i++) {
    //     hash_value = e1000_hash_mc_addr_generic(hw, mc_addr_list);
    //     hash_reg = (hash_value >> 5) & (hw->mac.mta_reg_count - 1);
    //     hash_bit = hash_value & 0x1F;

    //     hw->mac.mta_shadow[hash_reg] |= (1 << hash_bit);
    //     mc_addr_list += (ETH_ADDR_LEN);
    // }
    // e1000_mac_println!("mc_addr_count {}", mc_addr_count);

    for i in 0..mc_addr_count as usize {

        // e1000_mac_println!("{}: {:?}", i, &adapter.mta);
        // e1000_mac_println!("calling hash_mc_addr_generic");

        let hash_value = self::hash_mc_addr_generic(adapter, &adapter.mta[i * 6..(i + 1) * 6]);
        let hash_reg = (hash_value >> 5) & (adapter.hw.mac.mta_reg_count as u32 - 1);
        let hash_bit = hash_value & 0x1F;
        adapter.hw.mac.mta_shadow[hash_reg as usize] |= 1 << hash_bit;

        // e1000_mac_println!(
        //     "{}: Got hash value {} from {:?}",
        //     i,
        //     hash_value,
        //     &adapter.mta[i * 6..(i + 1) * 6]
        // );
    }


    // /* replace the entire MTA table */
    // for (i = hw->mac.mta_reg_count - 1; i >= 0; i--)
    //     E1000_WRITE_REG_ARRAY(hw, E1000_MTA, i, hw->mac.mta_shadow[i]);
    // E1000_WRITE_FLUSH(hw);

    let mut i = (adapter.hw.mac.mta_reg_count - 1) as isize;
    while i >= 0 {
        do_write_register_array(
            adapter,
            E1000_MTA,
            i as u32,
            adapter.hw.mac.mta_shadow[i as usize],
        );
        // e1000_mac_println!(
        //     "Writing mta {}: {}",
        //     i,
        //     adapter.hw.mac.mta_shadow[i as usize]
        // );
        i -= 1;
    }
    do_write_flush(adapter);
    Ok(())
}

pub fn get_speed_and_duplex_copper_generic(
    adapter: &mut Adapter,
    speed: &mut u16,
    duplex: &mut u16,
) -> AdResult {
    e1000_mac_println!();


    // status = E1000_READ_REG(hw, E1000_STATUS);
    // if (status & E1000_STATUS_SPEED_1000) {
    //     *speed = SPEED_1000;
    //     DEBUGOUT("1000 Mbs, ");
    // } else if (status & E1000_STATUS_SPEED_100) {
    //     *speed = SPEED_100;
    //     DEBUGOUT("100 Mbs, ");
    // } else {
    //     *speed = SPEED_10;
    //     DEBUGOUT("10 Mbs, ");
    // }

    let status: u32 = do_read_register(adapter, E1000_STATUS);

    if status & E1000_STATUS_SPEED_1000 != 0 {
        *speed = SPEED_1000;
        e1000_mac_println!("1000 Mbs");
    } else if status & E1000_STATUS_SPEED_100 != 0 {
        *speed = SPEED_100;
        e1000_mac_println!("100 Mbs");
    } else if status & E1000_STATUS_SPEED_10 != 0 {
        *speed = SPEED_10;
        e1000_mac_println!("10 Mbs");
    }

    // if (status & E1000_STATUS_FD) {
    //     *duplex = FULL_DUPLEX;
    //     DEBUGOUT("Full Duplex\n");
    // } else {
    //     *duplex = HALF_DUPLEX;
    //     DEBUGOUT("Half Duplex\n");
    // }

    if status & E1000_STATUS_FD != 0 {
        *duplex = FULL_DUPLEX;
        e1000_mac_println!("Full duplex");
    } else {
        *duplex = HALF_DUPLEX;
        e1000_mac_println!("Half duplex");
    }

    // return E1000_SUCCESS;
    Ok(())
}

pub fn get_speed_and_duplex_fiber_serdes_generic(
    adapter: &mut Adapter,
    arg2: &mut u16,
    arg3: &mut u16,
) -> AdResult {
    e1000_mac_println!();
    incomplete_return!();
}

pub fn check_for_serdes_link_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();
    incomplete_return!();
}

pub fn check_for_fiber_link_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();
    incomplete_return!();
}

pub fn check_for_copper_link_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();

    // struct e1000_mac_info *mac = &hw->mac;
    // s32 ret_val;
    // bool link;
    let mut link: bool = false;

    // DEBUGFUNC("e1000_check_for_copper_link");

    /* We only want to go out to the PHY registers to see if Auto-Neg
     * has completed and/or if our link status has changed.  The
     * get_link_status flag is set upon receiving a Link Status
     * Change or Rx Sequence Error interrupt.
     */
    // if (!mac->get_link_status)
    //     return E1000_SUCCESS;
    if !adapter.hw.mac.get_link_status {
        return Ok(());
    }

    /* First we want to see if the MII Status Register reports
     * link.  If so, then we want to get the current speed/duplex
     * of the PHY.
     */
    // ret_val = e1000_phy_has_link_generic(hw, 1, 0, &link);
    // if (ret_val)
    //     return ret_val;
    try!(e1000_phy::has_link_generic(adapter, 1, 0, &mut link));
    // if (!link)
    //     return E1000_SUCCESS; /* No link detected */
    if !link {
        e1000_mac_println!("No link detected");
        return Ok(());
    }
    // mac->get_link_status = FALSE;
    adapter.hw.mac.get_link_status = false;

    /* Check if there was DownShift, must be checked
     * immediately after link-up
     */
    // e1000_check_downshift_generic(hw);
    try!(e1000_phy::check_downshift_generic(adapter));

    /* If we are forcing speed/duplex, then we simply return since
     * we have already determined whether we have link or not.
     */
    // if (!mac->autoneg)
    //     return -E1000_ERR_CONFIG;
    if !adapter.hw.mac.autoneg {
        return Err("Config error".to_string());
    }

    /* Auto-Neg is enabled.  Auto Speed Detection takes care
     * of MAC speed/duplex configuration.  So we only need to
     * configure Collision Distance in the MAC.
     */
    // mac->ops.config_collision_dist(hw);
    try!(
        adapter
            .hw
            .mac
            .ops
            .config_collision_dist
            .ok_or("No function".to_string())
            .and_then(|f| {
                f(adapter);
                Ok(())
            })
    );

    /* Configure Flow Control now that Auto-Neg has completed.
     * First, we need to restore the desired flow control
     * settings because we may have had to re-autoneg with a
     * different link partner.
     */
    // ret_val = e1000_config_fc_after_link_up_generic(hw);
    // if (ret_val)
    //     DEBUGOUT("Error configuring flow control\n");

    try!(self::config_fc_after_link_up_generic(adapter));

    // return ret_val;
    Ok(())
}

pub fn setup_fiber_serdes_link_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();
    incomplete_return!();
}

pub fn setup_link_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();

    /* In the case of the phy reset being blocked, we already have a link.
     * We do not need to set it up again.
     */
    // if (hw->phy.ops.check_reset_block && hw->phy.ops.check_reset_block(hw))
    //     return E1000_SUCCESS;
    match adapter.check_reset_block() {
        Ok(true) => return Ok(()),
        Ok(false) => (),
        Err(e) => eprintln!(e),
    }

    /* If requested flow control is set to default, set flow control
     * based on the EEPROM flow control settings.
     */
    // if (hw->fc.requested_mode == e1000_fc_default) {
    //     ret_val = e1000_set_default_fc_generic(hw);
    //     if (ret_val)
    //         return ret_val;
    // }
    if adapter.hw.fc.requested_mode == FcMode::Default {
        incomplete_return!();
    }
    /* Save off the requested flow control mode for use later.  Depending
     * on the link partner's capabilities, we may or may not use this mode.
     */
    // hw->fc.current_mode = hw->fc.requested_mode;
    adapter.hw.fc.current_mode = adapter.hw.fc.requested_mode;

    // DEBUGOUT1("After fix-ups FlowControl is now = %x\n",
    // hw->fc.current_mode);
    e1000_mac_println!(
        "After fix-ups FlowControl is now = {:?}",
        adapter.hw.fc.current_mode
    );

    /* Call the necessary media_type subroutine to configure the link. */
    // ret_val = hw->mac.ops.setup_physical_interface(hw);
    // if (ret_val)
    //     return ret_val;
    if let Some(f) = adapter.hw.mac.ops.setup_physical_interface {
        try!(f(adapter));
    } else {
        return Err("Missing setup_physical_interface() function".to_string());
    }

    /* Initialize the flow control address, type, and PAUSE timer
     * registers to their default values.  This is done even if flow
     * control is disabled, because it does not hurt anything to
     * initialize these registers.
     */
    // DEBUGOUT("Initializing the Flow Control address, type and timer regs\n");
    // E1000_WRITE_REG(hw, E1000_FCT, FLOW_CONTROL_TYPE);
    // E1000_WRITE_REG(hw, E1000_FCAH, FLOW_CONTROL_ADDRESS_HIGH);
    // E1000_WRITE_REG(hw, E1000_FCAL, FLOW_CONTROL_ADDRESS_LOW);
    // E1000_WRITE_REG(hw, E1000_FCTTV, hw->fc.pause_time);

    do_write_register(adapter, E1000_FCT, FLOW_CONTROL_TYPE);
    do_write_register(adapter, E1000_FCAH, FLOW_CONTROL_ADDRESS_HIGH);
    do_write_register(adapter, E1000_FCAL, FLOW_CONTROL_ADDRESS_LOW);
    do_write_register(adapter, E1000_FCTTV, adapter.hw.fc.pause_time as u32);

    // return e1000_set_fc_watermarks_generic(hw);

    self::set_fc_watermarks_generic(adapter);

    // incomplete!();
    Ok(())
}

pub fn set_lan_id_multi_port_pci(adapter: &mut Adapter) {
    e1000_mac_println!();
    // 	struct e1000_bus_info *bus = &hw->bus;
    // 	u16 pci_header_type;
    // 	u32 status;

    // 	e1000_read_pci_cfg(hw, PCI_HEADER_TYPE_REGISTER, &pci_header_type);

    // 	if (pci_header_type & PCI_HEADER_TYPE_MULTIFUNC) {
    // 		status = E1000_READ_REG(hw, E1000_STATUS);
    // 		bus->func = (status & E1000_STATUS_FUNC_MASK)
    // 			    >> E1000_STATUS_FUNC_SHIFT;
    // 	} else {
    // 		bus->func = 0;
    // 	}
    // }

    let pci_header_type: u16 = adapter.dev.pci_read_config(PCI_HEADER_TYPE_REGISTER, 2) as u16;
    if btst!(pci_header_type, PCI_HEADER_TYPE_MULTIFUNC as u16) {
        let status = do_read_register(adapter, E1000_STATUS);
        adapter.hw.bus.func = ((status & E1000_STATUS_FUNC_MASK) >> E1000_STATUS_FUNC_SHIFT) as u16;
    } else {
        adapter.hw.bus.func = 0;
    }
}

pub fn set_lan_id_multi_port_pcie(adapter: &mut Adapter) {
    e1000_mac_println!();
    incomplete!();
}

pub fn validate_mdi_setting_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();
    incomplete_return!();
}

pub fn rar_set_generic(adapter: &mut Adapter, addr: &[u8], index: usize) -> AdResult {
    e1000_mac_println!();

    // u32 rar_low, rar_high;
    // DEBUGFUNC("e1000_rar_set_generic");

    /* HW expects these in little endian so we reverse the byte order
     * from network order (big endian) to little endian
     */
    // rar_low = ((u32) addr[0] | ((u32) addr[1] << 8) |
    //            ((u32) addr[2] << 16) | ((u32) addr[3] << 24));
    // rar_high = ((u32) addr[4] | ((u32) addr[5] << 8));

    let (rar_low, mut rar_high): (u32, u32) = {
        let low: u32 = (addr[0] as u32) | (addr[1] as u32) << 8 | (addr[2] as u32) << 16 |
            (addr[3] as u32) << 24;
        let high: u32 = (addr[4] as u32) | (addr[5] as u32) << 8;
        (low, high)
    };

    /* If MAC address zero, no need to set the AV bit */
    // if (rar_low || rar_high)
    //     rar_high |= E1000_RAH_AV;
    if rar_low != 0 || rar_high != 0 {
        rar_high |= E1000_RAH_AV;
    }

    /* Some bridges will combine consecutive 32-bit writes into
     * a single burst write, which will malfunction on some parts.
     * The flushes avoid this.
     */
    // E1000_WRITE_REG(hw, E1000_RAL(index), rar_low);
    // E1000_WRITE_FLUSH(hw);
    // E1000_WRITE_REG(hw, E1000_RAH(index), rar_high);
    // E1000_WRITE_FLUSH(hw);

    do_write_register(adapter, E1000_RAL(index), rar_low);
    do_write_flush(adapter);

    do_write_register(adapter, E1000_RAH(index), rar_high);
    do_write_flush(adapter);

    Ok(())
}

pub fn config_collision_dist_generic(adapter: &mut Adapter) {
    e1000_mac_println!();
    let mut tctl: u32;
    // u32 tctl;
    // DEBUGFUNC("e1000_config_collision_dist_generic");
    // tctl = E1000_READ_REG(hw, E1000_TCTL);
    // tctl &= ~E1000_TCTL_COLD;
    // tctl |= E1000_COLLISION_DISTANCE << E1000_COLD_SHIFT;
    // E1000_WRITE_REG(hw, E1000_TCTL, tctl);
    // E1000_WRITE_FLUSH(hw);
    tctl = do_read_register(adapter, E1000_TCTL);
    tctl &= !E1000_TCTL_COLD;
    tctl |= E1000_COLLISION_DISTANCE << E1000_COLD_SHIFT;
    do_write_register(adapter, E1000_TCTL, tctl);
    do_write_flush(adapter);
}

pub fn valid_led_default_generic(adapter: &mut Adapter, arg2: &mut [u16]) -> AdResult {
    e1000_mac_println!();
    incomplete_return!();
}

pub fn init_rx_addrs_generic(adapter: &mut Adapter, rar_count: usize) -> AdResult {
    e1000_mac_println!();
    // u32 i;
    // u8 mac_addr[ETH_ADDR_LEN] = {0};
    // DEBUGFUNC("e1000_init_rx_addrs_generic");
    let zero_addr: [u8; 6] = [0, 0, 0, 0, 0, 0];
    let hw_addr: [u8; 6] = adapter.hw.mac.addr;

    /* Setup the receive address */
    // DEBUGOUT("Programming MAC Address into RAR[0]\n");
    // hw->mac.ops.rar_set(hw, hw->mac.addr, 0);
    try!(adapter.rar_set(&hw_addr, 0));

    /* Zero out the other (rar_entry_count - 1) receive addresses */
    // DEBUGOUT1("Clearing RAR[1-%u]\n", rar_count-1);
    // for (i = 1; i < rar_count; i++)
    // 	hw->mac.ops.rar_set(hw, mac_addr, i);

    for i in 1..rar_count {
        try!(adapter.rar_set(&zero_addr, i));
    }

    Ok(())
}

pub fn pcix_mmrbc_workaround_generic(adapter: &mut Adapter) {
    e1000_mac_println!();
    // u16 cmd_mmrbc;
    // u16 pcix_cmd;
    // u16 pcix_stat_hi_word;
    // u16 stat_mmrbc;

    // DEBUGFUNC("e1000_pcix_mmrbc_workaround_generic");

    /* Workaround for PCI-X issue when BIOS sets MMRBC incorrectly */
    // if (hw->bus.type != e1000_bus_type_pcix)
    // 	return;
    if adapter.hw.bus.bustype != BusType::Pcix {
        return;
    }

    // e1000_read_pci_cfg(hw, PCIX_COMMAND_REGISTER, &pcix_cmd);
    // e1000_read_pci_cfg(hw, PCIX_STATUS_REGISTER_HI, &pcix_stat_hi_word);
    // cmd_mmrbc = (pcix_cmd & PCIX_COMMAND_MMRBC_MASK) >>
    // 	     PCIX_COMMAND_MMRBC_SHIFT;
    // stat_mmrbc = (pcix_stat_hi_word & PCIX_STATUS_HI_MMRBC_MASK) >>
    // 	      PCIX_STATUS_HI_MMRBC_SHIFT;
    // if (stat_mmrbc == PCIX_STATUS_HI_MMRBC_4K)
    // 	stat_mmrbc = PCIX_STATUS_HI_MMRBC_2K;
    // if (cmd_mmrbc > stat_mmrbc) {
    // 	pcix_cmd &= ~PCIX_COMMAND_MMRBC_MASK;
    // 	pcix_cmd |= stat_mmrbc << PCIX_COMMAND_MMRBC_SHIFT;
    // 	e1000_write_pci_cfg(hw, PCIX_COMMAND_REGISTER, &pcix_cmd);
    // }
    incomplete!();
}

pub fn set_fc_watermarks_generic(adapter: &mut Adapter) {
    e1000_mac_println!();
    // u32 fcrtl = 0, fcrth = 0;

    // DEBUGFUNC("e1000_set_fc_watermarks_generic");

    /* Set the flow control receive threshold registers.  Normally,
     * these registers will be set to a default threshold that may be
     * adjusted later by the driver's runtime code.  However, if the
     * ability to transmit pause frames is not enabled, then these
     * registers will be set to 0.
     */
    // if (hw->fc.current_mode & e1000_fc_tx_pause) {
    //     /* We need to set up the Receive Threshold high and low water
    //      * marks as well as (optionally) enabling the transmission of
    //      * XON frames.
    //      */
    //     fcrtl = hw->fc.low_water;
    //     if (hw->fc.send_xon)
    //         fcrtl |= E1000_FCRTL_XONE;

    //     fcrth = hw->fc.high_water;
    // }
    let mut fctrl = 0;
    let mut fctrh = 0;
    if btst!(adapter.hw.fc.current_mode as u32, FcMode::TxPause as u32) {
        fctrl = adapter.hw.fc.low_water;
        if adapter.hw.fc.send_xon {
            fctrl |= E1000_FCRTL_XONE;
        }
        fctrh = adapter.hw.fc.high_water;
    }
    // E1000_WRITE_REG(hw, E1000_FCRTL, fcrtl);
    // E1000_WRITE_REG(hw, E1000_FCRTH, fcrth);
    do_write_register(adapter, E1000_FCRTL, fctrl);
    do_write_register(adapter, E1000_FCRTH, fctrh);
    // return E1000_SUCCESS;
}

pub fn config_fc_after_link_up_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();
    // struct e1000_mac_info *mac = &hw->mac;
    // s32 ret_val = E1000_SUCCESS;
    // u32 pcs_status_reg, pcs_adv_reg, pcs_lp_ability_reg, pcs_ctrl_reg;
    // u16 mii_status_reg, mii_nway_adv_reg, mii_nway_lp_ability_reg;
    // u16 speed, duplex;

    // let mut pcs_status_reg;
    // let mut pcs_adv_reg;
    // let mut pcs_lp_ability_reg;
    // let mut pcs_ctrl_reg;
    let mut mii_status_reg = 0;
    let mut mii_nway_adv_reg = 0;
    let mut mii_nway_lp_ability_reg = 0;
    let mut speed = 0;
    let mut duplex = 0;

    // DEBUGFUNC("e1000_config_fc_after_link_up_generic");

    // /* Check for the case where we have fiber media and auto-neg failed
    //  * so we had to force link.  In this case, we need to force the
    //  * configuration of the MAC to match the "fc" parameter.
    //  */
    // if (mac->autoneg_failed) {
    //     if (hw->phy.media_type == e1000_media_type_fiber ||
    //         hw->phy.media_type == e1000_media_type_internal_serdes)
    //         ret_val = e1000_force_mac_fc_generic(hw);
    // } else {
    //     if (hw->phy.media_type == e1000_media_type_copper)
    //         ret_val = e1000_force_mac_fc_generic(hw);
    // }
    // if (ret_val) {
    //     DEBUGOUT("Error forcing flow control settings\n");
    //     return ret_val;
    // }
    if adapter.hw.mac.autoneg_failed {
        incomplete_return!();
    } else {
        if adapter.hw.phy.media_type == MediaType::Copper {
            try!(self::force_mac_fc_generic(adapter));
        }
    }

    /* Check for the case where we have copper media and auto-neg is
     * enabled.  In this case, we need to check and see if Auto-Neg
     * has completed, and if so, how the PHY and link partner has
     * flow control configured.
     */
    // if ((hw->phy.media_type == e1000_media_type_copper) && mac->autoneg) {
    if adapter.hw.phy.media_type == MediaType::Copper && adapter.hw.mac.autoneg {

        /* Read the MII Status Register and check to see if AutoNeg
         * has completed.  We read this twice because this reg has
         * some "sticky" (latched) bits.
         */
        //     ret_val = hw->phy.ops.read_reg(hw, PHY_STATUS, &mii_status_reg);
        //     if (ret_val)
        //         return ret_val;
        //     ret_val = hw->phy.ops.read_reg(hw, PHY_STATUS, &mii_status_reg);
        //     if (ret_val)
        //         return ret_val;
        try!(adapter.phy_read_reg(PHY_STATUS, &mut mii_status_reg));
        try!(adapter.phy_read_reg(PHY_STATUS, &mut mii_status_reg));

        //     if (!(mii_status_reg & MII_SR_AUTONEG_COMPLETE)) {
        //         DEBUGOUT("Copper PHY and Auto Neg has not completed.\n");
        //         return ret_val;
        //     }
        if !btst!(mii_status_reg, MII_SR_AUTONEG_COMPLETE as u16) {
            eprintln!("Copper PHY and Auto Neg has not completed");
            return Err(format!(
                "Copper PHY and Auto Neg has not completed (mii_status_reg = 0x{:x})",
                mii_status_reg
            ));
        }
        /* The AutoNeg process has completed, so we now need to
         * read both the Auto Negotiation Advertisement
         * Register (Address 4) and the Auto_Negotiation Base
         * Page Ability Register (Address 5) to determine how
         * flow control was negotiated.
         */
        //     ret_val = hw->phy.ops.read_reg(hw, PHY_AUTONEG_ADV,
        //     			       &mii_nway_adv_reg);
        //     if (ret_val)
        //         return ret_val;
        try!(adapter.phy_read_reg(PHY_AUTONEG_ADV, &mut mii_nway_adv_reg));
        //     ret_val = hw->phy.ops.read_reg(hw, PHY_LP_ABILITY,
        //     			       &mii_nway_lp_ability_reg);
        //     if (ret_val)
        //         return ret_val;
        try!(adapter.phy_read_reg(
            PHY_LP_ABILITY,
            &mut mii_nway_lp_ability_reg,
        ));

        /* Two bits in the Auto Negotiation Advertisement Register
         * (Address 4) and two bits in the Auto Negotiation Base
         * Page Ability Register (Address 5) determine flow control
         * for both the PHY and the link partner.  The following
         * table, taken out of the IEEE 802.3ab/D6.0 dated March 25,
         * 1999, describes these PAUSE resolution bits and how flow
         * control is determined based upon these settings.
         * NOTE:  DC = Don't Care
         *
         *   LOCAL DEVICE  |   LINK PARTNER
         * PAUSE | ASM_DIR | PAUSE | ASM_DIR | NIC Resolution
         *-------|---------|-------|---------|--------------------
         *   0   |    0    |  DC   |   DC    | e1000_fc_none
         *   0   |    1    |   0   |   DC    | e1000_fc_none
         *   0   |    1    |   1   |    0    | e1000_fc_none
         *   0   |    1    |   1   |    1    | e1000_fc_tx_pause
         *   1   |    0    |   0   |   DC    | e1000_fc_none
         *   1   |   DC    |   1   |   DC    | e1000_fc_full
         *   1   |    1    |   0   |    0    | e1000_fc_none
         *   1   |    1    |   0   |    1    | e1000_fc_rx_pause
         *
         * Are both PAUSE bits set to 1?  If so, this implies
         * Symmetric Flow Control is enabled at both ends.  The
         * ASM_DIR bits are irrelevant per the spec.
         *
         * For Symmetric Flow Control:
         *
         *   LOCAL DEVICE  |   LINK PARTNER
         * PAUSE | ASM_DIR | PAUSE | ASM_DIR | Result
         *-------|---------|-------|---------|--------------------
         *   1   |   DC    |   1   |   DC    | E1000_fc_full
         *
         */


        //     if ((mii_nway_adv_reg & NWAY_AR_PAUSE) &&
        //         (mii_nway_lp_ability_reg & NWAY_LPAR_PAUSE)) {
        if btst!(mii_nway_adv_reg, NWAY_AR_PAUSE as u16) &&
            btst!(mii_nway_lp_ability_reg, NWAY_LPAR_PAUSE as u16)
        {
            /* Now we need to check if the user selected Rx ONLY
             * of pause frames.  In this case, we had to advertise
             * FULL flow control because we could not advertise Rx
             * ONLY. Hence, we must now check to see if we need to
             * turn OFF the TRANSMISSION of PAUSE frames.
             */
            //         if (hw->fc.requested_mode == e1000_fc_full) {
            //     	hw->fc.current_mode = e1000_fc_full;
            //     	DEBUGOUT("Flow Control = FULL.\n");
            //         } else {
            //     	hw->fc.current_mode = e1000_fc_rx_pause;
            //     	DEBUGOUT("Flow Control = Rx PAUSE frames only.\n");
            //         }
            if adapter.hw.fc.requested_mode == FcMode::Full {
                adapter.hw.fc.current_mode = FcMode::Full;
                e1000_mac_println!("Flow Control = FULL");
            } else {
                adapter.hw.fc.current_mode = FcMode::RxPause;
                e1000_mac_println!("Flow Control = Rx PAUSE frames only");
            }
        }
        //     }
        //     /* For receiving PAUSE frames ONLY.
        //      *
        //      *   LOCAL DEVICE  |   LINK PARTNER
        //      * PAUSE | ASM_DIR | PAUSE | ASM_DIR | Result
        //      *-------|---------|-------|---------|--------------------
        //      *   0   |    1    |   1   |    1    | e1000_fc_tx_pause
        //      */
        //     else if (!(mii_nway_adv_reg & NWAY_AR_PAUSE) &&
        //     	 (mii_nway_adv_reg & NWAY_AR_ASM_DIR) &&
        //     	 (mii_nway_lp_ability_reg & NWAY_LPAR_PAUSE) &&
        //     	 (mii_nway_lp_ability_reg & NWAY_LPAR_ASM_DIR)) {
        //         hw->fc.current_mode = e1000_fc_tx_pause;
        //         DEBUGOUT("Flow Control = Tx PAUSE frames only.\n");
        //     }
        else if mii_nway_adv_reg & NWAY_AR_PAUSE as u16 == 0 &&
                   mii_nway_adv_reg & NWAY_AR_ASM_DIR as u16 > 0 &&
                   mii_nway_lp_ability_reg & NWAY_LPAR_PAUSE as u16 > 0 &&
                   mii_nway_lp_ability_reg & NWAY_LPAR_ASM_DIR as u16 > 0
        {
            adapter.hw.fc.current_mode = FcMode::TxPause;
            e1000_mac_println!("Flow Control = Tx PAUSE frames only");
        }
        /* For transmitting PAUSE frames ONLY.
         *
         *   LOCAL DEVICE  |   LINK PARTNER
         * PAUSE | ASM_DIR | PAUSE | ASM_DIR | Result
         *-------|---------|-------|---------|--------------------
         *   1   |    1    |   0   |    1    | e1000_fc_rx_pause
         */
        //     else if ((mii_nway_adv_reg & NWAY_AR_PAUSE) &&
        //     	 (mii_nway_adv_reg & NWAY_AR_ASM_DIR) &&
        //     	 !(mii_nway_lp_ability_reg & NWAY_LPAR_PAUSE) &&
        //     	 (mii_nway_lp_ability_reg & NWAY_LPAR_ASM_DIR)) {
        //         hw->fc.current_mode = e1000_fc_rx_pause;
        //         DEBUGOUT("Flow Control = Rx PAUSE frames only.\n");
        else if mii_nway_adv_reg & NWAY_AR_PAUSE as u16 > 0 &&
                   mii_nway_adv_reg & NWAY_AR_ASM_DIR as u16 > 0 &&
                   mii_nway_lp_ability_reg & NWAY_LPAR_PAUSE as u16 == 0 &&
                   mii_nway_lp_ability_reg & NWAY_LPAR_ASM_DIR as u16 > 0
        {
            adapter.hw.fc.current_mode = FcMode::RxPause;
            e1000_mac_println!("Flow Control = Rx PAUSE frames only");

        //     } else {
        /* Per the IEEE spec, at this point flow control
         * should be disabled.
         */
        //         hw->fc.current_mode = e1000_fc_none;
        //         DEBUGOUT("Flow Control = NONE.\n");
        //     }
        } else {
            adapter.hw.fc.current_mode = FcMode::None;
            e1000_mac_println!("Flow Control = NONE");
        }

        /* Now we need to do one last check...  If we auto-
         * negotiated to HALF DUPLEX, flow control should not be
         * enabled per IEEE 802.3 spec.
         */
        //     ret_val = mac->ops.get_link_up_info(hw, &speed, &duplex);
        //     if (ret_val) {
        //         DEBUGOUT("Error getting link speed and duplex\n");
        //         return ret_val;
        //     }
        try!(
            adapter
                .hw
                .mac
                .ops
                .get_link_up_info
                .ok_or("No function".to_string())
                .and_then(|f| f(adapter, &mut speed, &mut duplex))
        );

        //     if (duplex == HALF_DUPLEX)
        //         hw->fc.current_mode = e1000_fc_none;
        if duplex == HALF_DUPLEX as u16 {
            adapter.hw.fc.current_mode = FcMode::None;
        }
        /* Now we call a subroutine to actually force the MAC
         * controller to use the correct flow control settings.
         */
        //     ret_val = e1000_force_mac_fc_generic(hw);
        //     if (ret_val) {
        //         DEBUGOUT("Error forcing flow control settings\n");
        //         return ret_val;
        //     }
        // }
        try!(self::force_mac_fc_generic(adapter));
    }

    /* Check for the case where we have SerDes media and auto-neg is
     * enabled.  In this case, we need to check and see if Auto-Neg
     * has completed, and if so, how the PHY and link partner has
     * flow control configured.
     */
    // if ((hw->phy.media_type == e1000_media_type_internal_serdes) &&
    //     mac->autoneg) {
    if adapter.hw.phy.media_type == MediaType::InternalSerdes && adapter.hw.mac.autoneg {
        // Not on bhyve or Dell hardware
        incomplete_return!();
        //     /* Read the PCS_LSTS and check to see if AutoNeg
        //      * has completed.
        //      */
        //     pcs_status_reg = E1000_READ_REG(hw, E1000_PCS_LSTAT);

        //     if (!(pcs_status_reg & E1000_PCS_LSTS_AN_COMPLETE)) {
        //         DEBUGOUT("PCS Auto Neg has not completed.\n");
        //         return ret_val;
        //     }

        //     /* The AutoNeg process has completed, so we now need to
        //      * read both the Auto Negotiation Advertisement
        //      * Register (PCS_ANADV) and the Auto_Negotiation Base
        //      * Page Ability Register (PCS_LPAB) to determine how
        //      * flow control was negotiated.
        //      */
        //     pcs_adv_reg = E1000_READ_REG(hw, E1000_PCS_ANADV);
        //     pcs_lp_ability_reg = E1000_READ_REG(hw, E1000_PCS_LPAB);

        //     /* Two bits in the Auto Negotiation Advertisement Register
        //      * (PCS_ANADV) and two bits in the Auto Negotiation Base
        //      * Page Ability Register (PCS_LPAB) determine flow control
        //      * for both the PHY and the link partner.  The following
        //      * table, taken out of the IEEE 802.3ab/D6.0 dated March 25,
        //      * 1999, describes these PAUSE resolution bits and how flow
        //      * control is determined based upon these settings.
        //      * NOTE:  DC = Don't Care
        //      *
        //      *   LOCAL DEVICE  |   LINK PARTNER
        //      * PAUSE | ASM_DIR | PAUSE | ASM_DIR | NIC Resolution
        //      *-------|---------|-------|---------|--------------------
        //      *   0   |    0    |  DC   |   DC    | e1000_fc_none
        //      *   0   |    1    |   0   |   DC    | e1000_fc_none
        //      *   0   |    1    |   1   |    0    | e1000_fc_none
        //      *   0   |    1    |   1   |    1    | e1000_fc_tx_pause
        //      *   1   |    0    |   0   |   DC    | e1000_fc_none
        //      *   1   |   DC    |   1   |   DC    | e1000_fc_full
        //      *   1   |    1    |   0   |    0    | e1000_fc_none
        //      *   1   |    1    |   0   |    1    | e1000_fc_rx_pause
        //      *
        //      * Are both PAUSE bits set to 1?  If so, this implies
        //      * Symmetric Flow Control is enabled at both ends.  The
        //      * ASM_DIR bits are irrelevant per the spec.
        //      *
        //      * For Symmetric Flow Control:
        //      *
        //      *   LOCAL DEVICE  |   LINK PARTNER
        //      * PAUSE | ASM_DIR | PAUSE | ASM_DIR | Result
        //      *-------|---------|-------|---------|--------------------
        //      *   1   |   DC    |   1   |   DC    | e1000_fc_full
        //      *
        //      */
        //     if ((pcs_adv_reg & E1000_TXCW_PAUSE) &&
        //         (pcs_lp_ability_reg & E1000_TXCW_PAUSE)) {
        //         /* Now we need to check if the user selected Rx ONLY
        //          * of pause frames.  In this case, we had to advertise
        //          * FULL flow control because we could not advertise Rx
        //          * ONLY. Hence, we must now check to see if we need to
        //          * turn OFF the TRANSMISSION of PAUSE frames.
        //          */
        //         if (hw->fc.requested_mode == e1000_fc_full) {
        //     	hw->fc.current_mode = e1000_fc_full;
        //     	DEBUGOUT("Flow Control = FULL.\n");
        //         } else {
        //     	hw->fc.current_mode = e1000_fc_rx_pause;
        //     	DEBUGOUT("Flow Control = Rx PAUSE frames only.\n");
        //         }
        //     }
        //     /* For receiving PAUSE frames ONLY.
        //      *
        //      *   LOCAL DEVICE  |   LINK PARTNER
        //      * PAUSE | ASM_DIR | PAUSE | ASM_DIR | Result
        //      *-------|---------|-------|---------|--------------------
        //      *   0   |    1    |   1   |    1    | e1000_fc_tx_pause
        //      */
        //     else if (!(pcs_adv_reg & E1000_TXCW_PAUSE) &&
        //     	 (pcs_adv_reg & E1000_TXCW_ASM_DIR) &&
        //     	 (pcs_lp_ability_reg & E1000_TXCW_PAUSE) &&
        //     	 (pcs_lp_ability_reg & E1000_TXCW_ASM_DIR)) {
        //         hw->fc.current_mode = e1000_fc_tx_pause;
        //         DEBUGOUT("Flow Control = Tx PAUSE frames only.\n");
        //     }
        //     /* For transmitting PAUSE frames ONLY.
        //      *
        //      *   LOCAL DEVICE  |   LINK PARTNER
        //      * PAUSE | ASM_DIR | PAUSE | ASM_DIR | Result
        //      *-------|---------|-------|---------|--------------------
        //      *   1   |    1    |   0   |    1    | e1000_fc_rx_pause
        //      */
        //     else if ((pcs_adv_reg & E1000_TXCW_PAUSE) &&
        //     	 (pcs_adv_reg & E1000_TXCW_ASM_DIR) &&
        //     	 !(pcs_lp_ability_reg & E1000_TXCW_PAUSE) &&
        //     	 (pcs_lp_ability_reg & E1000_TXCW_ASM_DIR)) {
        //         hw->fc.current_mode = e1000_fc_rx_pause;
        //         DEBUGOUT("Flow Control = Rx PAUSE frames only.\n");
        //     } else {
        //         /* Per the IEEE spec, at this point flow control
        //          * should be disabled.
        //          */
        //         hw->fc.current_mode = e1000_fc_none;
        //         DEBUGOUT("Flow Control = NONE.\n");
        //     }

        //     /* Now we call a subroutine to actually force the MAC
        //      * controller to use the correct flow control settings.
        //      */
        //     pcs_ctrl_reg = E1000_READ_REG(hw, E1000_PCS_LCTL);
        //     pcs_ctrl_reg |= E1000_PCS_LCTL_FORCE_FCTRL;
        //     E1000_WRITE_REG(hw, E1000_PCS_LCTL, pcs_ctrl_reg);

        //     ret_val = e1000_force_mac_fc_generic(hw);
        //     if (ret_val) {
        //         DEBUGOUT("Error forcing flow control settings\n");
        //         return ret_val;
        //     }
        // }
    }

    // return E1000_SUCCESS;
    Ok(())
}

pub fn force_mac_fc_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();
    // u32 ctrl;

    // DEBUGFUNC("e1000_force_mac_fc_generic");

    // ctrl = E1000_READ_REG(hw, E1000_CTRL);

    let mut ctrl = do_read_register(adapter, E1000_CTRL);

    /* Because we didn't get link via the internal auto-negotiation
     * mechanism (we either forced link or we got link via PHY
     * auto-neg), we have to manually enable/disable transmit an
     * receive flow control.
     *
     * The "Case" statement below enables/disable flow control
     * according to the "hw->fc.current_mode" parameter.
     *
     * The possible values of the "fc" parameter are:
     *      0:  Flow control is completely disabled
     *      1:  Rx flow control is enabled (we can receive pause
     *          frames but not send pause frames).
     *      2:  Tx flow control is enabled (we can send pause frames
     *          frames but we do not receive pause frames).
     *      3:  Both Rx and Tx flow control (symmetric) is enabled.
     *  other:  No other values should be possible at this point.
     */
    // DEBUGOUT1("hw->fc.current_mode = %u\n", hw->fc.current_mode);

    // switch (hw->fc.current_mode) {
    //     case e1000_fc_none:
    //     ctrl &= (~(E1000_CTRL_TFCE | E1000_CTRL_RFCE));
    //     break;
    //     case e1000_fc_rx_pause:
    //     ctrl &= (~E1000_CTRL_TFCE);
    //     ctrl |= E1000_CTRL_RFCE;
    //     break;
    //     case e1000_fc_tx_pause:
    //     ctrl &= (~E1000_CTRL_RFCE);
    //     ctrl |= E1000_CTRL_TFCE;
    //     break;
    //     case e1000_fc_full:
    //     ctrl |= (E1000_CTRL_TFCE | E1000_CTRL_RFCE);
    //     break;
    //     default:
    //     DEBUGOUT("Flow control param set incorrectly\n");
    //     return -E1000_ERR_CONFIG;
    // }
    match adapter.hw.fc.current_mode {
        FcMode::None => ctrl &= !(E1000_CTRL_TFCE | E1000_CTRL_RFCE),
        FcMode::RxPause => {
            ctrl &= !E1000_CTRL_TFCE;
            ctrl |= E1000_CTRL_RFCE;
        }
        FcMode::TxPause => {
            ctrl &= !E1000_CTRL_RFCE;
            ctrl |= E1000_CTRL_TFCE;
        }
        FcMode::Full => ctrl |= E1000_CTRL_TFCE | E1000_CTRL_RFCE,
        _ => return Err("Flow control param set incorrectly".to_string()),
    };
    // E1000_WRITE_REG(hw, E1000_CTRL, ctrl);
    do_write_register(adapter, E1000_CTRL, ctrl);

    // return E1000_SUCCESS;
    Ok(())
}

pub fn clear_hw_cntrs_base_generic(adapter: &mut Adapter) {
    e1000_mac_println!();

    do_read_register(adapter, E1000_CRCERRS);
    do_read_register(adapter, E1000_SYMERRS);
    do_read_register(adapter, E1000_MPC);
    do_read_register(adapter, E1000_SCC);
    do_read_register(adapter, E1000_ECOL);
    do_read_register(adapter, E1000_MCC);
    do_read_register(adapter, E1000_LATECOL);
    do_read_register(adapter, E1000_COLC);
    do_read_register(adapter, E1000_DC);
    do_read_register(adapter, E1000_SEC);
    do_read_register(adapter, E1000_RLEC);
    do_read_register(adapter, E1000_XONRXC);
    do_read_register(adapter, E1000_XONTXC);
    do_read_register(adapter, E1000_XOFFRXC);
    do_read_register(adapter, E1000_XOFFTXC);
    do_read_register(adapter, E1000_FCRUC);
    do_read_register(adapter, E1000_GPRC);
    do_read_register(adapter, E1000_BPRC);
    do_read_register(adapter, E1000_MPRC);
    do_read_register(adapter, E1000_GPTC);
    do_read_register(adapter, E1000_GORCL);
    do_read_register(adapter, E1000_GORCH);
    do_read_register(adapter, E1000_GOTCL);
    do_read_register(adapter, E1000_GOTCH);
    do_read_register(adapter, E1000_RNBC);
    do_read_register(adapter, E1000_RUC);
    do_read_register(adapter, E1000_RFC);
    do_read_register(adapter, E1000_ROC);
    do_read_register(adapter, E1000_RJC);
    do_read_register(adapter, E1000_TORL);
    do_read_register(adapter, E1000_TORH);
    do_read_register(adapter, E1000_TOTL);
    do_read_register(adapter, E1000_TOTH);
    do_read_register(adapter, E1000_TPR);
    do_read_register(adapter, E1000_TPT);
    do_read_register(adapter, E1000_MPTC);
    do_read_register(adapter, E1000_BPTC);
}

/// e1000_set_lan_id_single_port - Set LAN id for a single port device
/// @hw: pointer to the HW structure
///
/// Sets the LAN function id to zero for a single port device.
pub fn set_lan_id_single_port(adapter: &mut Adapter) {
    e1000_mac_println!();
    // struct e1000_bus_info *bus = &hw->bus;
    // bus->func = 0;
    adapter.hw.bus.func = 0;
}


/// e1000_blink_led_generic - Blink LED
/// @hw: pointer to the HW structure
///
/// Blink the LEDs which are set to be on.
pub fn blink_led_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();
    // u32 ledctl_blink = 0;
    // u32 i;

    // DEBUGFUNC("e1000_blink_led_generic");

    // if (hw->phy.media_type == e1000_media_type_fiber) {
    //     /* always blink LED0 for PCI-E fiber */
    //     ledctl_blink = E1000_LEDCTL_LED0_BLINK |
    //     (E1000_LEDCTL_MODE_LED_ON << E1000_LEDCTL_LED0_MODE_SHIFT);
    // } else {
    //     /* Set the blink bit for each LED that's "on" (0x0E)
    //      * (or "off" if inverted) in ledctl_mode2.  The blink
    //      * logic in hardware only works when mode is set to "on"
    //      * so it must be changed accordingly when the mode is
    //      * "off" and inverted.
    //      */
    //     ledctl_blink = hw->mac.ledctl_mode2;
    //     for (i = 0; i < 32; i += 8) {
    //         u32 mode = (hw->mac.ledctl_mode2 >> i) &
    //     	E1000_LEDCTL_LED0_MODE_MASK;
    //         u32 led_default = hw->mac.ledctl_default >> i;

    //         if ((!(led_default & E1000_LEDCTL_LED0_IVRT) &&
    //     	 (mode == E1000_LEDCTL_MODE_LED_ON)) ||
    //     	((led_default & E1000_LEDCTL_LED0_IVRT) &&
    //     	 (mode == E1000_LEDCTL_MODE_LED_OFF))) {
    //     	ledctl_blink &=
    //     	    ~(E1000_LEDCTL_LED0_MODE_MASK << i);
    //     			                   ledctl_blink |= (E1000_LEDCTL_LED0_BLINK |
    //     					                    E1000_LEDCTL_MODE_LED_ON) << i;
    //     	    }
    //     	    }
    //         }

    //         E1000_WRITE_REG(hw, E1000_LEDCTL, ledctl_blink);

    //         return E1000_SUCCESS;
    incomplete_return!();
}

/// e1000_get_auto_rd_done_generic - Check for auto read completion
/// @hw: pointer to the HW structure
///
/// Check EEPROM for Auto Read done bit.
pub fn get_auto_rd_done_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();
    // while (i < AUTO_READ_DONE_TIMEOUT) {
    //     if (E1000_READ_REG(hw, E1000_EECD) & E1000_EECD_AUTO_RD)
    //         break;
    //     msec_delay(1);
    //     i++;
    // }
    let mut i = 0;
    while i < AUTO_READ_DONE_TIMEOUT {
        if btst!(adapter.read_register(E1000_EECD), E1000_EECD_AUTO_RD) {
            break;
        }
        do_msec_delay(1);
        i += 1;
    }

    // if (i == AUTO_READ_DONE_TIMEOUT) {
    //     DEBUGOUT("Auto read by HW from NVM has not completed.\n");
    //     return -E1000_ERR_RESET;
    // }
    // return E1000_SUCCESS;
    if i == AUTO_READ_DONE_TIMEOUT {
        Err("Auto read by HW from NVM has not completed".to_string())
    } else {
        Ok(())
    }
}


/// e1000_disable_pcie_master_generic - Disables PCI-express master access
/// @hw: pointer to the HW structure
///
/// Returns E1000_SUCCESS if successful, else returns -10
/// (-E1000_ERR_MASTER_REQUESTS_PENDING) if master disable bit has not caused
/// the master requests to be disabled.
///
/// Disables PCI-Express master access and verifies there are no pending
/// requests.
pub fn disable_pcie_master_generic(adapter: &mut Adapter) -> AdResult {
    e1000_mac_println!();

    // u32 ctrl;
    // s32 timeout = MASTER_DISABLE_TIMEOUT;
    let mut ctrl: u32;
    let mut timeout: u32 = MASTER_DISABLE_TIMEOUT;

    // DEBUGFUNC("e1000_disable_pcie_master_generic");

    // if (hw->bus.type != e1000_bus_type_pci_express)
    //     return E1000_SUCCESS;
    if adapter.hw.bus.bustype != BusType::Pci_express {
        return Ok(());
    }

    // ctrl = E1000_READ_REG(hw, E1000_CTRL);
    // ctrl |= E1000_CTRL_GIO_MASTER_DISABLE;
    // E1000_WRITE_REG(hw, E1000_CTRL, ctrl);
    adapter.set_register_bit(E1000_CTRL, E1000_CTRL_GIO_MASTER_DISABLE);

    // while (timeout) {
    //     if (!(E1000_READ_REG(hw, E1000_STATUS) &
    //           E1000_STATUS_GIO_MASTER_ENABLE) ||
    //         E1000_REMOVED(hw->hw_addr))
    //         break;
    //     usec_delay(100);
    //     timeout--;
    // }

    while timeout > 0 {
        // E1000_REMOVED(x) is defined to NOP
        if !btst!(
            adapter.read_register(E1000_STATUS),
            E1000_STATUS_GIO_MASTER_ENABLE
        )
        {
            break;
        }
        do_usec_delay(100);
        timeout -= 1;
    }

    // if (!timeout) {
    //     DEBUGOUT("Master requests are pending.\n");
    //     return -E1000_ERR_MASTER_REQUESTS_PENDING;
    // }
    if timeout == 0 {
        e1000_mac_println!("Master requests are pending");
        return Err("Master requests are pending".to_string());
    }
    // return E1000_SUCCESS;
    Ok(())
}

/// e1000_set_pcie_no_snoop_generic - Set PCI-express capabilities
/// @hw: pointer to the HW structure
/// @no_snoop: bitmap of snoop events
///
/// Set the PCI-express register to snoop for events enabled in 'no_snoop'.
pub fn set_pcie_no_snoop_generic(adapter: &mut Adapter, no_snoop: u32) {
    e1000_mac_println!();
    // u32 gcr;

    // DEBUGFUNC("e1000_set_pcie_no_snoop_generic");

    // if (hw->bus.type != e1000_bus_type_pci_express)
    //     return;
    if adapter.hw.bus.bustype != BusType::Pci_express {
        return;
    }

    // if (no_snoop) {
    //     gcr = E1000_READ_REG(hw, E1000_GCR);
    //     gcr &= ~(PCIE_NO_SNOOP_ALL);
    //     gcr |= no_snoop;
    //     E1000_WRITE_REG(hw, E1000_GCR, gcr);
    // }
    if no_snoop != 0 {
        let mut gcr = adapter.read_register(E1000_GCR);
        gcr &= !PCIE_NO_SNOOP_ALL;
        gcr |= no_snoop;
        adapter.write_register(E1000_GCR, gcr);
    }
}
