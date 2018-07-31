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

    adapter.hw.phy.media_type = match adapter.hw.device_id as u32 {
        E1000_DEV_ID_82545EM_FIBER
        | E1000_DEV_ID_82545GM_FIBER
        | E1000_DEV_ID_82546EB_FIBER
        | E1000_DEV_ID_82546GB_FIBER => MediaType::Fiber,
        E1000_DEV_ID_82545GM_SERDES | E1000_DEV_ID_82546GB_SERDES => MediaType::InternalSerdes,
        _ => MediaType::Copper,
    };

    /* Set mta register count */
    adapter.hw.mac.mta_reg_count = 128;

    /* Set rar entry count */
    adapter.hw.mac.rar_entry_count = E1000_RAR_ENTRIES as u16;

    /* Function pointers */

    /* bus type/speed/width */
    adapter.hw.mac.ops.get_bus_info = Some(e1000_mac::get_bus_info_pci_generic);

    /* function id */
    adapter.hw.mac.ops.set_lan_id = Some(e1000_mac::set_lan_id_multi_port_pci);

    /* reset */
    adapter.hw.mac.ops.reset_hw = Some(self::reset_hw);

    /* hw initialization */
    adapter.hw.mac.ops.init_hw = Some(self::init_hw);

    /* link setup */
    adapter.hw.mac.ops.setup_link = Some(e1000_mac::setup_link_generic);

    /* physical interface setup */
    adapter.hw.mac.ops.setup_physical_interface = match adapter.hw.phy.media_type {
        MediaType::Copper => Some(e1000_phy::setup_copper_link_generic),
        _ => Some(e1000_mac::setup_fiber_serdes_link_generic),
    };

    /* check for link */
    adapter.hw.mac.ops.check_for_link = match adapter.hw.phy.media_type {
        MediaType::Copper => Some(e1000_mac::check_for_copper_link_generic),
        MediaType::Fiber => Some(e1000_mac::check_for_fiber_link_generic),
        MediaType::InternalSerdes => Some(e1000_mac::check_for_serdes_link_generic),
        _ => return Err("No function for check_for_link".into()),
    };

    /* link info */
    adapter.hw.mac.ops.get_link_up_info = match adapter.hw.phy.media_type {
        MediaType::Copper => Some(e1000_mac::get_speed_and_duplex_copper_generic),
        _ => Some(e1000_mac::get_speed_and_duplex_fiber_serdes_generic),
    };

    /* multicast address update */
    adapter.hw.mac.ops.update_mc_addr_list = Some(e1000_mac::update_mc_addr_list_generic);

    /* writing VFTA */
    adapter.hw.mac.ops.write_vfta = Some(e1000_mac::write_vfta_generic);

    /* clearing VFTA */
    adapter.hw.mac.ops.clear_vfta = Some(e1000_mac::clear_vfta_generic);

    /* read mac address */
    adapter.hw.mac.ops.read_mac_addr = Some(self::read_mac_addr);

    /* ID LED init */
    adapter.hw.mac.ops.id_led_init = Some(e1000_mac::id_led_init_generic);

    /* setup LED */
    adapter.hw.mac.ops.setup_led = Some(e1000_mac::setup_led_generic);

    /* cleanup LED */
    adapter.hw.mac.ops.cleanup_led = Some(e1000_mac::cleanup_led_generic);

    /* turn on/off LED */
    adapter.hw.mac.ops.led_on = Some(e1000_mac::led_on_generic);

    adapter.hw.mac.ops.led_off = Some(e1000_mac::led_off_generic);

    /* clear hardware counters */
    adapter.hw.mac.ops.clear_hw_cntrs = Some(self::clear_hw_cntrs);

    Ok(())
}

pub fn init_phy_params(adapter: &mut Adapter) -> AdResult {
    e1000_println!();

    adapter.hw.phy.addr = 1;
    adapter.hw.phy.autoneg_mask = AUTONEG_ADVERTISE_SPEED_DEFAULT as u16;
    adapter.hw.phy.reset_delay_us = 10000;
    adapter.hw.phy.phy_type = PhyType::Type_m88;

    /* Function Pointers */
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

    try!(e1000_phy::get_phy_id(adapter));

    /* Verify phy id */
    match adapter.hw.mac.mac_type {
        MacType::Mac_82540
        | MacType::Mac_82545
        | MacType::Mac_82545_rev_3
        | MacType::Mac_82546
        | MacType::Mac_82546_rev_3 => match adapter.hw.phy.id {
            M88E1011_I_PHY_ID => Ok(()),
            _ => Err("Could not verify phy id".into()),
        },
        _ => Err("Could not find mac type for match phy id".into()),
    }
}

pub fn init_nvm_params(adapter: &mut Adapter) -> AdResult {
    e1000_println!();

    let eecd = do_read_register(adapter, E1000_EECD);
    e1000_println!("register EECD=0x{:x}", eecd);

    e1000_println!("eecd:            {:032b}", eecd);
    e1000_println!("E1000_EECD_SIZE: {:032b}", E1000_EECD_SIZE);

    adapter.hw.nvm.nvm_type = NvmType::EepromMicrowire;
    adapter.hw.nvm.delay_usec = 50;
    adapter.hw.nvm.opcode_bits = 3;

    let nvm: &mut NvmInfo = &mut adapter.hw.nvm;

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

    /* Function Pointers */
    nvm.ops.acquire = Some(e1000_nvm::acquire_nvm_generic);
    nvm.ops.read = Some(e1000_nvm::read_nvm_microwire);
    nvm.ops.release = Some(e1000_nvm::release_nvm_generic);
    nvm.ops.update = Some(e1000_nvm::update_nvm_checksum_generic);
    nvm.ops.valid_led_default = Some(e1000_mac::valid_led_default_generic);
    nvm.ops.validate = Some(e1000_nvm::validate_nvm_checksum_generic);
    nvm.ops.write = Some(e1000_nvm::write_nvm_microcode);

    Ok(())
}

pub fn read_mac_addr(adapter: &mut Adapter) -> AdResult {
    e1000_println!();

    let mut offset;
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
    if adapter.hw.bus.func == E1000_FUNC_1 as u16 {
        adapter.hw.mac.perm_addr[5] ^= 1;
    }

    for i in 0usize..kernel::sys::iflib_sys::ETH_ADDR_LEN as usize {
        adapter.hw.mac.addr[i] = adapter.hw.mac.perm_addr[i];
    }
    Ok(())
}

pub fn init_hw(adapter: &mut Adapter) -> AdResult {
    e1000_println!();

    /* Initialize identification LED */
    if let Err(e) = adapter
        .hw
        .mac
        .ops
        .id_led_init
        .ok_or("No function: id_led_init".to_string())
    {
        eprintln!("{:?}", e);
        eprintln!("Failed to initialize identification LED (IGNORE)");
    }

    /* Disabling VLAN filtering */
    if adapter.hw.mac.mac_type < MacType::Mac_82545_rev_3 {
        do_write_register(adapter, E1000_VET, 0);
    }

    if let Some(f) = adapter.hw.mac.ops.clear_vfta {
        f(adapter);
    }

    /* Setup the receive address. */
    let rar_entry_count = adapter.hw.mac.rar_entry_count as usize;
    try!(e1000_mac::init_rx_addrs_generic(adapter, rar_entry_count));

    /* Zero out the Multicast HASH table */
    for i in 0..adapter.hw.mac.mta_reg_count {
        do_write_register_array(adapter, E1000_MTA, i as u32, 0);
        do_write_flush(adapter);
    }

    if adapter.hw.mac.mac_type < MacType::Mac_82545_rev_3 {
        e1000_mac::pcix_mmrbc_workaround_generic(adapter);
    }

    /* Setup link and flow control */
    if let Some(f) = adapter.hw.mac.ops.setup_link {
        try!(f(adapter));
    }
    let mut txdctl = do_read_register(adapter, E1000_TXCTL(0));
    txdctl = txdctl & !E1000_TXDCTL_WTHRESH | E1000_TXDCTL_FULL_TX_DESC_WB;
    do_write_register(adapter, E1000_TXCTL(0), txdctl);

    /*
     * Clear all of the statistics registers (clear on read).  It is
     * important that we do this after we have tried to establish link
     * because the symbol error count will increment wildly if there
     * is no link.
     */
    self::clear_hw_cntrs(adapter);

    Ok(())
}

pub fn reset_hw(adapter: &mut Adapter) -> AdResult {
    e1000_println!();

    e1000_println!("Masking off all interrupts");
    do_write_register(adapter, E1000_IMC, 0xffffffff);
    do_write_register(adapter, E1000_RCTL, 0);
    do_write_register(adapter, E1000_TCTL, E1000_TCTL_PSP);
    do_write_flush(adapter);

    /*
     * Delay to allow any outstanding PCI transactions to complete
     * before resetting the device.
     */
    do_msec_delay(10);

    let ctrl: u32 = do_read_register(adapter, E1000_CTRL);

    e1000_println!("Issuing a global reset to 82540/82545/82546 MAC");

    match adapter.hw.mac.mac_type {
        MacType::Mac_82545_rev_3 | MacType::Mac_82546 => {
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
    do_msec_delay(5);

    /* Disable HW ARPs on ASF enabled adapters */
    let mut manc: u32 = do_read_register(adapter, E1000_MANC);
    manc &= !E1000_MANC_ARP_EN;
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
