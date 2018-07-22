
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
use e1000_mac;
use e1000_osdep::*;
use e1000_regs::*;
use e1000_phy;
use e1000_nvm;

pub fn init_function_pointers(adapter: &mut Adapter) -> AdResult {

    e1000_println!();

    adapter.hw.mac.ops.init_params = Some(init_mac_params);
    adapter.hw.nvm.ops.init_params = Some(init_nvm_params);
    adapter.hw.phy.ops.init_params = Some(init_phy_params);

    Ok(())
}


pub fn init_mac_params(adapter: &mut Adapter) -> AdResult {

    e1000_println!();

    // 	struct e1000_mac_info *mac = &hw->mac;
    // 	s32 ret_val = E1000_SUCCESS;

    // 	DEBUGFUNC("e1000_init_mac_params_82540");

    // 	/* Set media type */
    // 	switch (hw->device_id) {
    // 	case E1000_DEV_ID_82545EM_FIBER:
    // 	case E1000_DEV_ID_82545GM_FIBER:
    // 	case E1000_DEV_ID_82546EB_FIBER:
    // 	case E1000_DEV_ID_82546GB_FIBER:
    // 		hw->phy.media_type = e1000_media_type_fiber;
    // 		break;
    // 	case E1000_DEV_ID_82545GM_SERDES:
    // 	case E1000_DEV_ID_82546GB_SERDES:
    // 		hw->phy.media_type = e1000_media_type_internal_serdes;
    // 		break;
    // 	default:
    // 		hw->phy.media_type = e1000_media_type_copper;
    // 		break;
    // 	}

    adapter.hw.phy.media_type = match adapter.hw.device_id as u32 {
        E1000_DEV_ID_82545EM_FIBER |
        E1000_DEV_ID_82545GM_FIBER |
        E1000_DEV_ID_82546EB_FIBER |
        E1000_DEV_ID_82546GB_FIBER => MediaType::Fiber,
        E1000_DEV_ID_82545GM_SERDES |
        E1000_DEV_ID_82546GB_SERDES => MediaType::InternalSerdes,
        _ => MediaType::Copper,
    };

    /* Set mta register count */
    // 	mac->mta_reg_count = 128;
    adapter.hw.mac.mta_reg_count = 128;

    /* Set rar entry count */
    // 	mac->rar_entry_count = E1000_RAR_ENTRIES;
    adapter.hw.mac.rar_entry_count = E1000_RAR_ENTRIES as u16;



    /* Function pointers */

    /* bus type/speed/width */
    // 	mac->ops.get_bus_info = e1000_get_bus_info_pci_generic;
    adapter.hw.mac.ops.get_bus_info = Some(e1000_mac::get_bus_info_pci_generic);


    /* function id */
    // 	mac->ops.set_lan_id = e1000_set_lan_id_multi_port_pci;
    adapter.hw.mac.ops.set_lan_id = Some(e1000_mac::set_lan_id_multi_port_pci);

    /* reset */
    // 	mac->ops.reset_hw = e1000_reset_hw_82540;
    adapter.hw.mac.ops.reset_hw = Some(self::reset_hw);

    /* hw initialization */
    // 	mac->ops.init_hw = e1000_init_hw_82540;
    adapter.hw.mac.ops.init_hw = Some(self::init_hw);

    /* link setup */
    // 	mac->ops.setup_link = e1000_setup_link_generic;
    adapter.hw.mac.ops.setup_link = Some(e1000_mac::setup_link_generic);

    /* physical interface setup */
    // 	mac->ops.setup_physical_interface =
    // 		(hw->phy.media_type == e1000_media_type_copper)
    // 			? e1000_setup_copper_link_82540
    // 			: e1000_setup_fiber_serdes_link_82540;
    adapter.hw.mac.ops.setup_physical_interface = match adapter.hw.phy.media_type {
        MediaType::Copper => Some(e1000_phy::setup_copper_link_generic),
        _ => Some(e1000_mac::setup_fiber_serdes_link_generic),
    };

    /* check for link */
    // 	switch (hw->phy.media_type) {
    // 	case e1000_media_type_copper:
    // 		mac->ops.check_for_link = e1000_check_for_copper_link_generic;
    // 		break;
    // 	case e1000_media_type_fiber:
    // 		mac->ops.check_for_link = e1000_check_for_fiber_link_generic;
    // 		break;
    // 	case e1000_media_type_internal_serdes:
    // 		mac->ops.check_for_link = e1000_check_for_serdes_link_generic;
    // 		break;
    // 	default:
    // 		ret_val = -E1000_ERR_CONFIG;
    // 		goto out;
    // 		break;
    // 	}

    adapter.hw.mac.ops.check_for_link = match adapter.hw.phy.media_type {
        MediaType::Copper => Some(e1000_mac::check_for_copper_link_generic),
        MediaType::Fiber => Some(e1000_mac::check_for_fiber_link_generic),
        MediaType::InternalSerdes => Some(e1000_mac::check_for_serdes_link_generic),
        _ => return Err("No function for check_for_link".into()),
    };


    /* link info */
    // 	mac->ops.get_link_up_info =
    // 		(hw->phy.media_type == e1000_media_type_copper)
    // 			? e1000_get_speed_and_duplex_copper_generic
    // 			: e1000_get_speed_and_duplex_fiber_serdes_generic;
    adapter.hw.mac.ops.get_link_up_info = match adapter.hw.phy.media_type {
        MediaType::Copper => Some(e1000_mac::get_speed_and_duplex_copper_generic),
        _ => Some(e1000_mac::get_speed_and_duplex_fiber_serdes_generic),
    };

    /* multicast address update */
    // 	mac->ops.update_mc_addr_list = e1000_update_mc_addr_list_generic;
    adapter.hw.mac.ops.update_mc_addr_list = Some(e1000_mac::update_mc_addr_list_generic);

    /* writing VFTA */
    // 	mac->ops.write_vfta = e1000_write_vfta_generic;
    adapter.hw.mac.ops.write_vfta = Some(e1000_mac::write_vfta_generic);

    /* clearing VFTA */
    // 	mac->ops.clear_vfta = e1000_clear_vfta_generic;
    adapter.hw.mac.ops.clear_vfta = Some(e1000_mac::clear_vfta_generic);

    /* read mac address */
    // 	mac->ops.read_mac_addr = e1000_read_mac_addr_82540;
    adapter.hw.mac.ops.read_mac_addr = Some(self::read_mac_addr);

    /* ID LED init */
    // 	mac->ops.id_led_init = e1000_id_led_init_generic;
    adapter.hw.mac.ops.id_led_init = Some(e1000_mac::id_led_init_generic);

    /* setup LED */
    // 	mac->ops.setup_led = e1000_setup_led_generic;
    adapter.hw.mac.ops.setup_led = Some(e1000_mac::setup_led_generic);

    /* cleanup LED */
    // 	mac->ops.cleanup_led = e1000_cleanup_led_generic;
    adapter.hw.mac.ops.cleanup_led = Some(e1000_mac::cleanup_led_generic);

    /* turn on/off LED */
    // 	mac->ops.led_on = e1000_led_on_generic;
    adapter.hw.mac.ops.led_on = Some(e1000_mac::led_on_generic);

    // 	mac->ops.led_off = e1000_led_off_generic;
    adapter.hw.mac.ops.led_off = Some(e1000_mac::led_off_generic);

    /* clear hardware counters */
    // 	mac->ops.clear_hw_cntrs = e1000_clear_hw_cntrs_82540;
    adapter.hw.mac.ops.clear_hw_cntrs = Some(self::clear_hw_cntrs);

    Ok(())
    // This function finished.
}


pub fn init_phy_params(adapter: &mut Adapter) -> AdResult {
    e1000_println!();

    // struct e1000_phy_info *phy = &hw->phy;
    // s32 ret_val;

    // phy->addr		= 1;
    // phy->autoneg_mask	= AUTONEG_ADVERTISE_SPEED_DEFAULT;
    // phy->reset_delay_us	= 10000;
    // phy->type		= e1000_phy_m88;

    adapter.hw.phy.addr = 1;
    adapter.hw.phy.autoneg_mask = AUTONEG_ADVERTISE_SPEED_DEFAULT as u16;
    adapter.hw.phy.reset_delay_us = 10000;
    adapter.hw.phy.phy_type = PhyType::Type_m88;

    // /* Function Pointers */
    // phy->ops.check_polarity	= e1000_check_polarity_m88;
    // phy->ops.commit		= e1000_phy_sw_reset_generic;
    // phy->ops.force_speed_duplex = e1000_phy_force_speed_duplex_m88;
    // phy->ops.get_cable_length = e1000_get_cable_length_m88;
    // phy->ops.get_cfg_done	= e1000_get_cfg_done_generic;
    // phy->ops.read_reg	= e1000_read_phy_reg_m88;
    // phy->ops.reset		= e1000_phy_hw_reset_generic;
    // phy->ops.write_reg	= e1000_write_phy_reg_m88;
    // phy->ops.get_info	= e1000_get_phy_info_m88;
    // phy->ops.power_up	= e1000_power_up_phy_copper;
    // phy->ops.power_down	= e1000_power_down_phy_copper_82540;

    adapter.hw.phy.ops.check_polarity = Some(e1000_phy::check_polarity_m88);
    adapter.hw.phy.ops.commit = Some(e1000_phy::phy_sw_reset_generic);
    adapter.hw.phy.ops.force_speed_duplex = Some(e1000_phy::phy_force_speed_duplex_m88);
    adapter.hw.phy.ops.get_cable_length = Some(e1000_phy::get_cable_length_m88);
    adapter.hw.phy.ops.get_cfg_done = Some(e1000_phy::get_cfg_done_generic);
    adapter.hw.phy.ops.read_reg = Some(e1000_phy::read_phy_reg_m88);
    adapter.hw.phy.ops.reset = Some(e1000_phy::phy_hw_reset_generic);
    adapter.hw.phy.ops.write_reg = Some(e1000_phy::write_phy_reg_m88);
    adapter.hw.phy.ops.get_info = Some(e1000_phy::get_phy_info_m88);
    adapter.hw.phy.ops.power_up = Some(e1000_phy::power_up_phy_copper);
    adapter.hw.phy.ops.power_down = Some(self::power_down_phy_copper);

    // ret_val = e1000_get_phy_id(hw);
    // if (ret_val)
    //     goto out;

    try!(e1000_phy::get_phy_id(adapter));


    // /* Verify phy id */
    // switch (hw->mac.type) {
    //     case e1000_82540:
    //     case e1000_82545:
    //     case e1000_82545_rev_3:
    //     case e1000_82546:
    //     case e1000_82546_rev_3:
    //     if (phy->id == M88E1011_I_PHY_ID)
    //         break;
    //     /* Fall Through */
    //     default:
    //     ret_val = -E1000_ERR_PHY;
    //     goto out;
    //     break;
    // }

    match adapter.hw.mac.mac_type {
        MacType::Mac_82540 |
        MacType::Mac_82545 |
        MacType::Mac_82545_rev_3 |
        MacType::Mac_82546 |
        MacType::Mac_82546_rev_3 => {
            match adapter.hw.phy.id {
                M88E1011_I_PHY_ID => Ok(()),
                _ => Err("Could not verify phy id".into()),
            }
        }
        _ => Err("Could not find mac type for match phy id".into()),
    }
}

pub fn init_nvm_params(adapter: &mut Adapter) -> AdResult {
    e1000_println!();
    // struct e1000_nvm_info *nvm = &hw->nvm;
    // u32 eecd = E1000_READ_REG(hw, E1000_EECD);

    let eecd = do_read_register(adapter, E1000_EECD);
    e1000_println!("register EECD=0x{:x}", eecd);

    e1000_println!("eecd:            {:032b}", eecd);
    e1000_println!("E1000_EECD_SIZE: {:032b}", E1000_EECD_SIZE);

    // DEBUGFUNC("e1000_init_nvm_params_82540");

    // nvm->type = e1000_nvm_eeprom_microwire;
    // nvm->delay_usec = 50;
    // nvm->opcode_bits = 3;

    adapter.hw.nvm.nvm_type = NvmType::EepromMicrowire;
    adapter.hw.nvm.delay_usec = 50;
    adapter.hw.nvm.opcode_bits = 3;

    // switch (nvm->override) {
    // case e1000_nvm_override_microwire_large:
    // 	nvm->address_bits = 8;
    // 	nvm->word_size = 256;
    // 	break;
    // case e1000_nvm_override_microwire_small:
    // 	nvm->address_bits = 6;
    // 	nvm->word_size = 64;
    // 	break;
    // default:
    // 	nvm->address_bits = eecd & E1000_EECD_SIZE ? 8 : 6;
    // 	nvm->word_size = eecd & E1000_EECD_SIZE ? 256 : 64;
    // 	break;
    // }

    let nvm: &mut NvmInfo = &mut adapter.hw.nvm;

    // override is set where?
    match nvm.nvmoverride {
        NvmOverride::MicrowireLarge => {
            nvm.address_bits = 8;
            nvm.word_size = 256;
        }
        NvmOverride::MicrowireSmall => {
            nvm.address_bits = 6;
            nvm.word_size = 64;
        }
        _ => {
            nvm.address_bits = match eecd & E1000_EECD_SIZE {
                0 => 6,
                _ => 8,
            };
            nvm.word_size = match eecd & E1000_EECD_SIZE {
                0 => 64,
                _ => 256,
            };
        }
    }

    // /* Function Pointers */
    // nvm->ops.acquire	= e1000_acquire_nvm_generic;
    // nvm->ops.read		= e1000_read_nvm_microwire;
    // nvm->ops.release	= e1000_release_nvm_generic;
    // nvm->ops.update		= e1000_update_nvm_checksum_generic;
    // nvm->ops.valid_led_default = e1000_valid_led_default_generic;
    // nvm->ops.validate	= e1000_validate_nvm_checksum_generic;
    // nvm->ops.write		= e1000_write_nvm_microwire;

    nvm.ops.acquire = Some(e1000_nvm::acquire_nvm_generic);
    nvm.ops.read = Some(e1000_nvm::read_nvm_microwire);
    nvm.ops.release = Some(e1000_nvm::release_nvm_generic);
    nvm.ops.update = Some(e1000_nvm::update_nvm_checksum_generic);
    nvm.ops.valid_led_default = Some(e1000_mac::valid_led_default_generic);
    nvm.ops.validate = Some(e1000_nvm::validate_nvm_checksum_generic);
    nvm.ops.write = Some(e1000_nvm::write_nvm_microcode);

    // return E1000_SUCCESS;
    Ok(())
}

pub fn read_mac_addr(adapter: &mut Adapter) -> AdResult {
    e1000_println!();

    // for (i = 0; i < ETH_ADDR_LEN; i += 2) {
    // 	offset = i >> 1;
    // 	ret_val = hw->nvm.ops.read(hw, offset, 1, &nvm_data);
    // 	if (ret_val) {
    // 		DEBUGOUT("NVM Read Error\n");
    // 		goto out;
    // 	}
    // 	hw->mac.perm_addr[i] = (u8)(nvm_data & 0xFF);
    // 	hw->mac.perm_addr[i+1] = (u8)(nvm_data >> 8);
    // }

    let mut offset;
    // let mut ret = 0;
    let mut nvm_data: [u16; 1] = [0u16; 1];
    for i in (0usize..kernel::sys::iflib_sys::ETH_ADDR_LEN as usize).step_by(2) {
        offset = i >> 1;

        if let Some(read) = adapter.hw.nvm.ops.read {
            try!(read(adapter, offset as u16, 1, &mut nvm_data));
        } else {
            return Err("No function".into());
        }

        adapter.hw.mac.perm_addr[i] = (nvm_data[0] & 0xff) as u8;
        adapter.hw.mac.perm_addr[i + 1] = (nvm_data[0] >> 8) as u8;
    }
    /* Flip last bit of mac address if we're on second port */
    // if (hw->bus.func == E1000_FUNC_1)
    // 	hw->mac.perm_addr[5] ^= 1;
    if adapter.hw.bus.func == E1000_FUNC_1 as u16 {
        adapter.hw.mac.perm_addr[5] ^= 1;
    }

    // for (i = 0; i < ETH_ADDR_LEN; i++)
    // 	hw->mac.addr[i] = hw->mac.perm_addr[i];
    for i in 0usize..kernel::sys::iflib_sys::ETH_ADDR_LEN as usize {
        adapter.hw.mac.addr[i] = adapter.hw.mac.perm_addr[i];
    }
    Ok(())
}

pub fn init_hw(adapter: &mut Adapter) -> AdResult {
    e1000_println!();

    // /* Initialize identification LED */
    // ret_val = mac->ops.id_led_init(hw);
    // if (ret_val) {
    //     DEBUGOUT("Error initializing identification LED\n");
    //     /* This is not fatal and we should not stop init due to this */
    // }

    /* Disabling VLAN filtering */
    // DEBUGOUT("Initializing the IEEE VLAN\n");
    // if (mac->type < e1000_82545_rev_3)
    //     E1000_WRITE_REG(hw, E1000_VET, 0);
    if adapter.hw.mac.mac_type < MacType::Mac_82545_rev_3 {
        do_write_register(adapter, E1000_VET, 0);
    }

    // mac->ops.clear_vfta(hw);
    if let Some(f) = adapter.hw.mac.ops.clear_vfta {
        f(adapter);
    }

    /* Setup the receive address. */
    // e1000_init_rx_addrs_generic(hw, mac->rar_entry_count);
    let rar_entry_count = adapter.hw.mac.rar_entry_count as usize;
    try!(e1000_mac::init_rx_addrs_generic(adapter, rar_entry_count));

    /* Zero out the Multicast HASH table */
    //     DEBUGOUT("Zeroing the MTA\n");
    //     for (i = 0; i < mac->mta_reg_count; i++) {
    //         E1000_WRITE_REG_ARRAY(hw, E1000_MTA, i, 0);
    //         /*
    //          * Avoid back to back register writes by adding the register
    //          * read (flush).  This is to protect against some strange
    //          * bridge configurations that may issue Memory Write Block
    //          * (MWB) to our register space.  The *_rev_3 hardware at
    //          * least doesn't respond correctly to every other dword in an
    //          * MWB to our register space.
    //          */
    //         E1000_WRITE_FLUSH(hw);
    //     }
    for i in 0..adapter.hw.mac.mta_reg_count {
        do_write_register_array(adapter, E1000_MTA, i as u32, 0);
        do_write_flush(adapter);
    }

    //     if (mac->type < e1000_82545_rev_3)
    //         e1000_pcix_mmrbc_workaround_generic(hw);
    if adapter.hw.mac.mac_type < MacType::Mac_82545_rev_3 {
        e1000_mac::pcix_mmrbc_workaround_generic(adapter);
    }

    /* Setup link and flow control */
    // ret_val = mac->ops.setup_link(hw);
    if let Some(f) = adapter.hw.mac.ops.setup_link {
        try!(f(adapter));
    }
    // txdctl = E1000_READ_REG(hw, E1000_TXDCTL(0));
    // txdctl = (txdctl & ~E1000_TXDCTL_WTHRESH) |
    // E1000_TXDCTL_FULL_TX_DESC_WB;
    // E1000_WRITE_REG(hw, E1000_TXDCTL(0), txdctl);
    let mut txdctl = do_read_register(adapter, E1000_TXCTL(0));
    txdctl = txdctl & !E1000_TXDCTL_WTHRESH | E1000_TXDCTL_FULL_TX_DESC_WB;
    do_write_register(adapter, E1000_TXCTL(0), txdctl);

    /*
     * Clear all of the statistics registers (clear on read).  It is
     * important that we do this after we have tried to establish link
     * because the symbol error count will increment wildly if there
     * is no link.
     */
    // e1000_clear_hw_cntrs_82540(hw);
    self::clear_hw_cntrs(adapter);

    // if ((hw->device_id == E1000_DEV_ID_82546GB_QUAD_COPPER) ||
    //     (hw->device_id == E1000_DEV_ID_82546GB_QUAD_COPPER_KSP3)) {
    //     ctrl_ext = E1000_READ_REG(hw, E1000_CTRL_EXT);
    //     /*
    //      * Relaxed ordering must be disabled to avoid a parity
    //      * error crash in a PCI slot.
    //      */
    //     ctrl_ext |= E1000_CTRL_EXT_RO_DIS;
    //     E1000_WRITE_REG(hw, E1000_CTRL_EXT, ctrl_ext);
    // }

    Ok(())
}

pub fn reset_hw(adapter: &mut Adapter) -> AdResult {
    e1000_println!();

    // u32 ctrl, manc;
    // s32 ret_val = E1000_SUCCESS;

    // DEBUGFUNC("e1000_reset_hw_82540");

    // DEBUGOUT("Masking off all interrupts\n");

    e1000_println!("Masking off all interrupts");

    // E1000_WRITE_REG(hw, E1000_IMC, 0xFFFFFFFF);
    do_write_register(adapter, E1000_IMC, 0xffffffff);

    // E1000_WRITE_REG(hw, E1000_RCTL, 0);
    do_write_register(adapter, E1000_RCTL, 0);

    // E1000_WRITE_REG(hw, E1000_TCTL, E1000_TCTL_PSP);
    do_write_register(adapter, E1000_TCTL, E1000_TCTL_PSP);

    // E1000_WRITE_FLUSH(hw);
    do_write_flush(adapter);

    /*
     * Delay to allow any outstanding PCI transactions to complete
     * before resetting the device.
     */
    // msec_delay(10);
    do_msec_delay(10);

    // ctrl = E1000_READ_REG(hw, E1000_CTRL);
    let ctrl: u32 = do_read_register(adapter, E1000_CTRL);

    // DEBUGOUT("Issuing a global reset to 82540/82545/82546 MAC\n");
    // switch (hw->mac.type) {
    // case e1000_82545_rev_3:
    // case e1000_82546_rev_3:
    // 	E1000_WRITE_REG(hw, E1000_CTRL_DUP, ctrl | E1000_CTRL_RST);
    // 	break;
    // default:
    /*
     * These controllers can't ack the 64-bit write when
     * issuing the reset, so we use IO-mapping as a
     * workaround to issue the reset.
     */
    // 	E1000_WRITE_REG_IO(hw, E1000_CTRL, ctrl | E1000_CTRL_RST);
    // 	break;
    // }
    e1000_println!("Issuing a global reset to 82540/82545/82546 MAC");

    match adapter.hw.mac.mac_type {
        MacType::Mac_82545_rev_3 |
        MacType::Mac_82546 => {
            do_write_register(adapter, E1000_CTRL_DUP, ctrl | E1000_CTRL_RST);
        }
        _ => {
            /*
             * These controllers can't ack the 64-bit write when
             * issuing the reset, so we use IO-mapping as a
             * workaround to issue the reset.
             */
            do_write_register_io(adapter, E1000_CTRL, ctrl | E1000_CTRL_RST);
        }
    }

    /* Wait for EEPROM reload */
    // msec_delay(5);
    do_msec_delay(5);

    /* Disable HW ARPs on ASF enabled adapters */
    // manc = E1000_READ_REG(hw, E1000_MANC);
    // manc &= ~E1000_MANC_ARP_EN;
    let mut manc: u32 = do_read_register(adapter, E1000_MANC);
    manc &= !E1000_MANC_ARP_EN;
    // E1000_WRITE_REG(hw, E1000_MANC, manc);
    // E1000_WRITE_REG(hw, E1000_IMC, 0xffffffff);
    // E1000_READ_REG(hw, E1000_ICR);
    do_write_register(adapter, E1000_MANC, manc);
    do_write_register(adapter, E1000_IMC, 0xffffffff);
    do_read_register(adapter, E1000_ICR);

    Ok(())
}

pub fn clear_hw_cntrs(adapter: &mut Adapter) -> AdResult {
    e1000_println!();

    e1000_mac::clear_hw_cntrs_base_generic(adapter);

    do_read_register(adapter, E1000_PRC64);
    do_read_register(adapter, E1000_PRC127);
    do_read_register(adapter, E1000_PRC255);
    do_read_register(adapter, E1000_PRC511);
    do_read_register(adapter, E1000_PRC1023);
    do_read_register(adapter, E1000_PRC1522);
    do_read_register(adapter, E1000_PTC64);
    do_read_register(adapter, E1000_PTC127);
    do_read_register(adapter, E1000_PTC255);
    do_read_register(adapter, E1000_PTC511);
    do_read_register(adapter, E1000_PTC1023);
    do_read_register(adapter, E1000_PTC1522);
    do_read_register(adapter, E1000_ALGNERRC);
    do_read_register(adapter, E1000_RXERRC);
    do_read_register(adapter, E1000_TNCRS);
    do_read_register(adapter, E1000_CEXTERR);
    do_read_register(adapter, E1000_TSCTC);
    do_read_register(adapter, E1000_TSCTFC);
    do_read_register(adapter, E1000_MGTPRC);
    do_read_register(adapter, E1000_MGTPDC);
    do_read_register(adapter, E1000_MGTPTC);
    Ok(())
}

pub fn power_down_phy_copper(adapter: &mut Adapter) {
    e1000_println!();
    incomplete!();
}
