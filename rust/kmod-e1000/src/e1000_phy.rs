
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
use e1000_mac;

pub fn copper_link_autoneg(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();

    let mut phy_ctrl: u16 = 0;

    let read_reg = try!(adapter.hw.phy.ops.read_reg.ok_or("No function".to_string()));
    let write_reg = try!(adapter.hw.phy.ops.write_reg.ok_or(
        "No function".to_string(),
    ));

    /* Perform some bounds checking on the autoneg advertisement
     * parameter.
     */
    adapter.hw.phy.autoneg_advertised &= adapter.hw.phy.autoneg_mask;

    /* If autoneg_advertised is zero, we assume it was not defaulted
     * by the calling code so we set to advertise full capability.
     */
    if adapter.hw.phy.autoneg_advertised == 0 {
        adapter.hw.phy.autoneg_advertised = adapter.hw.phy.autoneg_mask;
    }

    e1000_phy_println!("Reconfiguring auto-neg advertisement params");
    try!(setup_autoneg(adapter));
    e1000_phy_println!("Restarting Auto-Neg");

    /* Restart auto-negotiation by setting the Auto Neg Enable bit and
     * the Auto Neg Restart bit in the PHY control register.
     */
    try!(read_reg(adapter, PHY_CONTROL, &mut phy_ctrl));

    phy_ctrl |= MII_CR_AUTO_NEG_EN | MII_CR_RESTART_AUTO_NEG;
    try!(write_reg(adapter, PHY_CONTROL, phy_ctrl));

    /* Does the user want to wait for Auto-Neg to complete here, or
     * check at a later time (for example, callback routine).
     */
    if adapter.hw.phy.autoneg_wait_to_complete {
        try!(wait_autoneg(adapter));
    }
    adapter.hw.mac.get_link_status = true;

    Ok(())
}

pub fn wait_autoneg(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();

    let mut phy_status: u16 = 0;
    /* Break after autoneg completes or PHY_AUTO_NEG_LIMIT expires. */
    if let Some(read_reg) = adapter.hw.phy.ops.read_reg {
        for i in 0..PHY_AUTO_NEG_LIMIT {
            try!(read_reg(adapter, PHY_STATUS, &mut phy_status));
            try!(read_reg(adapter, PHY_STATUS, &mut phy_status));
            if btst!(phy_status, MII_SR_AUTONEG_COMPLETE) {
                break;
            }
            do_msec_delay(100);
        }
    }
    /* PHY_AUTO_NEG_TIME expiration doesn't guarantee auto-negotiation
     * has completed.
     */
    Ok(())
}

pub fn setup_autoneg(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();

    let mut mii_autoneg_adv_reg: u16 = 0;
    let mut mii_1000t_ctrl_reg: u16 = 0;

    adapter.hw.phy.autoneg_advertised &= adapter.hw.phy.autoneg_mask;

    let read_reg = try!(adapter.hw.phy.ops.read_reg.ok_or("No function".to_string()));
    let write_reg = try!(adapter.hw.phy.ops.write_reg.ok_or(
        "No function".to_string(),
    ));

    /* Read the MII Auto-Neg Advertisement Register (Address 4). */
    try!(read_reg(adapter, PHY_AUTONEG_ADV, &mut mii_autoneg_adv_reg));

    if adapter.hw.phy.autoneg_mask & ADVERTISE_1000_FULL != 0 {
        /* Read the MII 1000Base-T Control Register (Address 9). */
        try!(read_reg(adapter, PHY_1000T_CTRL, &mut mii_1000t_ctrl_reg));
    }
    /* Need to parse both autoneg_advertised and fc and set up
     * the appropriate PHY registers.  First we will parse for
     * autoneg_advertised software override.  Since we can advertise
     * a plethora of combinations, we need to check each bit
     * individually.
     */

    /* First we clear all the 10/100 mb speed bits in the Auto-Neg
     * Advertisement Register (Address 4) and the 1000 mb speed bits in
     * the  1000Base-T Control Register (Address 9).
     */
    mii_autoneg_adv_reg &= !(NWAY_AR_100TX_FD_CAPS | NWAY_AR_100TX_HD_CAPS | NWAY_AR_10T_FD_CAPS |
                                 NWAY_AR_10T_HD_CAPS);
    mii_1000t_ctrl_reg &= !(CR_1000T_HD_CAPS | CR_1000T_FD_CAPS);

    /* Do we want to advertise 10 Mb Half Duplex? */
    if adapter.hw.phy.autoneg_advertised & ADVERTISE_10_HALF != 0 {
        e1000_phy_println!("Advertise 10mb Half duplex");
        mii_autoneg_adv_reg |= NWAY_AR_10T_HD_CAPS;
    }

    /* Do we want to advertise 10 Mb Full Duplex? */
    if adapter.hw.phy.autoneg_advertised & ADVERTISE_10_FULL != 0 {
        e1000_phy_println!("Advertise 10mb Full duplex");
        mii_autoneg_adv_reg |= NWAY_AR_10T_FD_CAPS;
    }

    /* Do we want to advertise 100 Mb Half Duplex? */
    if adapter.hw.phy.autoneg_advertised & ADVERTISE_100_HALF != 0 {
        e1000_phy_println!("Advertise 100mb Half duplex");
        mii_autoneg_adv_reg |= NWAY_AR_100TX_HD_CAPS;
    }

    /* Do we want to advertise 100 Mb Full Duplex? */
    if adapter.hw.phy.autoneg_advertised & ADVERTISE_100_FULL != 0 {
        e1000_phy_println!("Advertise 100mb Full duplex");
        mii_autoneg_adv_reg |= NWAY_AR_100TX_FD_CAPS;
    }

    /* We do not allow the Phy to advertise 1000 Mb Half Duplex */
    if adapter.hw.phy.autoneg_advertised & ADVERTISE_1000_HALF != 0 {
        e1000_phy_println!("Advertise 1000mb half duplex request denied");
    }

    /* Do we want to advertise 1000 Mb Full Duplex? */
    if adapter.hw.phy.autoneg_advertised & ADVERTISE_1000_FULL != 0 {
        e1000_phy_println!("Advertise 1000mb Full duplex");
        mii_1000t_ctrl_reg |= CR_1000T_FD_CAPS;
    }

    /* Check for a software override of the flow control settings, and
     * setup the PHY advertisement registers accordingly.  If
     * auto-negotiation is enabled, then software will have to set the
     * "PAUSE" bits to the correct value in the Auto-Negotiation
     * Advertisement Register (PHY_AUTONEG_ADV) and re-start auto-
     * negotiation.
     *
     * The possible values of the "fc" parameter are:
     *      0:  Flow control is completely disabled
     *      1:  Rx flow control is enabled (we can receive pause frames
     *          but not send pause frames).
     *      2:  Tx flow control is enabled (we can send pause frames
     *          but we do not support receiving pause frames).
     *      3:  Both Rx and Tx flow control (symmetric) are enabled.
     *  other:  No software override.  The flow control configuration
     *          in the EEPROM is used.
     */
    match adapter.hw.fc.current_mode {
        FcMode::None => {
            /* Flow control (Rx & Tx) is completely disabled by a
             * software over-ride.
             */
            mii_autoneg_adv_reg &= !(NWAY_AR_ASM_DIR | NWAY_AR_PAUSE);
        }
        FcMode::RxPause => {
            /* Rx Flow control is enabled, and Tx Flow control is
             * disabled, by a software over-ride.
             *
             * Since there really isn't a way to advertise that we are
             * capable of Rx Pause ONLY, we will advertise that we
             * support both symmetric and asymmetric Rx PAUSE.  Later
             * (in e1000_config_fc_after_link_up) we will disable the
             * hw's ability to send PAUSE frames.
             */
            mii_autoneg_adv_reg |= NWAY_AR_ASM_DIR | NWAY_AR_PAUSE;
        }
        FcMode::TxPause => {
            /* Tx Flow control is enabled, and Rx Flow control is
             * disabled, by a software over-ride.
             */
            mii_autoneg_adv_reg |= NWAY_AR_ASM_DIR;
            mii_autoneg_adv_reg &= !NWAY_AR_PAUSE;
        }
        FcMode::Full => {
            /* Flow control (both Rx and Tx) is enabled by a software
             * over-ride.
             */
            mii_autoneg_adv_reg |= NWAY_AR_ASM_DIR | NWAY_AR_PAUSE;
        }
        _ => {
            return Err("Flow control param set incorrectly".to_string());
        }
    }
    try!(write_reg(adapter, PHY_AUTONEG_ADV, mii_autoneg_adv_reg));

    e1000_phy_println!("Auto-Neg Advertising 0x{:x}", mii_autoneg_adv_reg);

    if adapter.hw.phy.autoneg_mask & ADVERTISE_1000_FULL != 0 {
        e1000_phy_println!("Auto-Neg Advertising 1000 Full");
        try!(write_reg(adapter, PHY_1000T_CTRL, mii_1000t_ctrl_reg));
    }
    Ok(())
}

pub fn setup_copper_link_generic(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();

    if adapter.hw.mac.autoneg {
        /* Setup autoneg and flow control advertisement and perform
         * autonegotiation.
         */
        try!(self::copper_link_autoneg(adapter));
    } else {
        /* PHY will be set to 10H, 10F, 100H or 100F
         * depending on user settings.
         */
        e1000_phy_println!("Forcing speed and duplex");
        try!(
            adapter
                .hw
                .phy
                .ops
                .force_speed_duplex
                .ok_or("No function".to_string())
                .and_then(|f| f(adapter))
        );
    }
    /* Check link status. Wait up to 100 microseconds for link to become
     * valid.
     */
    let mut link = false;
    try!(has_link_generic(
        adapter,
        COPPER_LINK_UP_LIMIT,
        10,
        &mut link,
    ));

    if link {
        e1000_println!("Valid link established");
        let f = try!(adapter.hw.mac.ops.config_collision_dist.ok_or(
            "No function".to_string(),
        ));
        f(adapter);
        e1000_mac::config_fc_after_link_up_generic(adapter)
    } else {
        e1000_println!("Unable to establish link");
        Ok(())
    }
}

/// Fatal on error
pub fn get_phy_id(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();

    let mut retry_count = 0;
    let mut phy_id: u16 = 0;

    if let Some(read) = adapter.hw.phy.ops.read_reg {
        while retry_count < 2 {

            try!(read(adapter, PHY_ID1, &mut phy_id));
            e1000_println!("Got raw PHY_ID1 = 0x{:x}", phy_id);
            adapter.hw.phy.id = (phy_id as u32) << 16;
            do_usec_delay(20);

            try!(read(adapter, PHY_ID2, &mut phy_id));
            e1000_println!("Got raw PHY_ID2 = 0x{:x}", phy_id);
            adapter.hw.phy.id |= (phy_id as u32) & PHY_REVISION_MASK;
            adapter.hw.phy.revision = (phy_id as u32) & !PHY_REVISION_MASK;

            if phy_id != 0 && phy_id as u32 != PHY_REVISION_MASK {
                return Ok(());
            }
            retry_count += 1;
        }
    }
    Ok(())
}

pub fn power_up_phy_copper(adapter: &mut Adapter) {
    e1000_phy_println!();
    incomplete!();
}

pub fn get_phy_info_m88(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();

    let mut phy_data: u16 = 0;
    let mut link: bool = false;
    let read_reg = try!(adapter.hw.phy.ops.read_reg.ok_or("No function".to_string()));

    if adapter.hw.phy.media_type != MediaType::Copper {
        return Err("Phy info is only valid for copper media".to_string());
    }

    try!(self::has_link_generic(adapter, 1, 0, &mut link));
    if !link {
        return Err("Phy info is only valid if link is up".to_string());
    }

    try!(read_reg(adapter, M88E1000_PHY_SPEC_CTRL, &mut phy_data));
    adapter.hw.phy.polarity_correction = phy_data & M88E1000_PSCR_POLARITY_REVERSAL != 0;

    try!(self::check_polarity_m88(adapter));
    try!(read_reg(adapter, M88E1000_PHY_SPEC_STATUS, &mut phy_data));

    adapter.hw.phy.is_mdix = phy_data & M88E1000_PSSR_MDIX != 0;

    if phy_data & M88E1000_PSSR_SPEED == M88E1000_PSSR_1000MBS {
        try!(
            adapter
                .hw
                .phy
                .ops
                .get_cable_length
                .ok_or("No function".to_string())
                .and_then(|f| f(adapter))
        );
        try!(read_reg(adapter, PHY_1000T_STATUS, &mut phy_data));
        adapter.hw.phy.local_rx = match phy_data & SR_1000T_LOCAL_RX_STATUS {
            x if x != 0 => GbRxStatus::Ok,
            _ => GbRxStatus::NotOk,
        };
        adapter.hw.phy.remote_rx = match phy_data & SR_1000T_REMOTE_RX_STATUS {
            x if x != 0 => GbRxStatus::Ok,
            _ => GbRxStatus::NotOk,
        };
    } else {
        adapter.hw.phy.cable_length = E1000_CABLE_LENGTH_UNDEFINED;
        adapter.hw.phy.local_rx = GbRxStatus::Undefined;
        adapter.hw.phy.remote_rx = GbRxStatus::Undefined;
    }

    Ok(())
}

pub fn write_phy_reg_m88(adapter: &mut Adapter, offset: u32, data: u16) -> AdResult {
    e1000_verbose_println!();

    if let (Some(acquire), Some(release)) =
        (adapter.hw.phy.ops.acquire, adapter.hw.phy.ops.release)
    {
        try!(acquire(adapter));
        try!(self::write_phy_reg_mdic(
            adapter,
            MAX_PHY_REG_ADDRESS & offset,
            data,
        ));
        try!(release(adapter));
    }
    Ok(())
}

pub fn read_phy_reg_m88(adapter: &mut Adapter, offset: u32, data: &mut u16) -> AdResult {
    e1000_verbose_println!();

    if let Some(acquire) = adapter.hw.phy.ops.acquire {
        try!(acquire(adapter));
    }

    let res = read_phy_reg_mdic(adapter, MAX_PHY_REG_ADDRESS & offset, data);
    e1000_phy_println!(
        "read register 0x{:x} to 0x{:x}",
        MAX_PHY_REG_ADDRESS & offset,
        *data
    );

    if let Some(release) = adapter.hw.phy.ops.release {
        try!(release(adapter));
    }
    res
}

pub fn write_phy_reg_mdic(adapter: &mut Adapter, offset: u32, data: u16) -> AdResult {
    e1000_verbose_println!();

    if offset > MAX_PHY_REG_ADDRESS {
        return Err(format!("PHY address {} is out of range", offset));
    }
    /* Set up Op-code, Phy Address, and register offset in the MDI
     * Control register.  The MAC will take care of interfacing with the
     * PHY to retrieve the desired data.
     */
    let mut mdic: u32 = (data as u32) | (offset << E1000_MDIC_REG_SHIFT) |
        (adapter.hw.phy.addr << E1000_MDIC_PHY_SHIFT) | E1000_MDIC_OP_WRITE;

    do_write_register(adapter, E1000_MDIC, mdic);

    /* Poll the ready bit to see if the MDI read completed
     * Increasing the time out as testing showed failures with
     * the lower time out
     */
    for i in 0..(E1000_GEN_POLL_TIMEOUT * 3) {
        do_usec_delay(50);
        mdic = do_read_register(adapter, E1000_MDIC);
        if mdic & E1000_MDIC_READY != 0 {
            break;
        }
    }

    if mdic & E1000_MDIC_READY == 0 {
        return Err("MDI write did not complete".to_string());
    }

    if mdic & E1000_MDIC_ERROR != 0 {
        return Err("MDI error".to_string());
    }

    if (mdic & E1000_MDIC_REG_MASK) >> E1000_MDIC_REG_SHIFT != offset {
        return Err(format!(
                "MDI write offset error - requested {}, returned {}",
                offset,
                (mdic & E1000_MDIC_REG_MASK) >> E1000_MDIC_REG_SHIFT,
        ));
    }

    /* Allow some time after each MDIC transaction to avoid
     * reading duplicate data in the next MDIC transaction.
     */
    if adapter.hw.mac.mac_type == MacType::Mac_pch2lan {
        do_usec_delay(100);
    }

    Ok(())
}

///  __e1000_read_phy_reg_hv -  Read HV PHY register
///  @hw: pointer to the HW structure
///  @offset: register offset to be read
///  @data: pointer to the read data
///  @locked: semaphore has already been acquired or not
///
///  Acquires semaphore, if necessary, then reads the PHY register at offset
///  and stores the retrieved information in data.  Release any acquired
///  semaphore before exiting.
pub fn __read_phy_reg_hv(
    adapter: &mut Adapter,
    offset: u32,
    data: &mut u16,
    locked: bool,
    page_set: bool,
) -> AdResult {
    /// Fatal on error
    e1000_verbose_println!();

    fn release_if_locked(adapter: &mut Adapter, locked: bool) -> AdResult {
        if !locked {
            adapter.phy_release()
        } else {
            Ok(())
        }
    };

    let mut page: u16 = BM_PHY_REG_PAGE(offset);
    let reg: u16 = BM_PHY_REG_NUM(offset);
    let phy_addr: u32 = get_phy_addr_for_hv_page(page as u32);
    adapter.hw.phy.addr = phy_addr;

    if !locked {
        try!(adapter.phy_acquire());
    }

    /* Page 800 works differently than the rest so it has its own func */
    if page == BM_WUC_PAGE as u16 {
        let res = access_phy_wakeup_reg_bm(adapter, offset, data, true, page_set);
        return release_if_locked(adapter, locked).and(res);
    }

    if page > 0 && page < HV_INTC_FC_PAGE_START as u16 {
        let res = access_phy_debug_regs_hv(adapter, offset, data, true);
        return release_if_locked(adapter, locked).and(res);
    }

    if !page_set {
        if page == HV_INTC_FC_PAGE_START as u16 {
            page = 0;
        }
        if reg > MAX_PHY_MULTI_PAGE_REG as u16 {
            let res = set_page_igp(adapter, page << IGP_PAGE_SHIFT);
            adapter.hw.phy.addr = phy_addr;
            if res.is_err() {
                return release_if_locked(adapter, locked).and(res);
            }
        }
    }

    let res = read_phy_reg_mdic(adapter, (reg as u32) & MAX_PHY_REG_ADDRESS, data);
    if res.is_err() {
        e1000_println!("read_phy_reg_mdic() error");
    }
    e1000_verbose_println!(
        "Read PHY page {} (or 0x{:x} shifted) reg 0x{:x}, data 0x{:x}",
        page,
        page << IGP_PAGE_SHIFT,
        reg, data
    );
    release_if_locked(adapter, locked).and(res)
}

///  e1000_read_phy_reg_hv -  Read HV PHY register
///  @hw: pointer to the HW structure
///  @offset: register offset to be read
///  @data: pointer to the read data
///
///  Acquires semaphore then reads the PHY register at offset and stores
///  the retrieved information in data.  Release the acquired semaphore
///  before exiting.
pub fn read_phy_reg_hv(adapter: &mut Adapter, offset: u32, data: &mut u16) -> AdResult {
    __read_phy_reg_hv(adapter, offset, data, false, false)
}

///  e1000_read_phy_reg_hv_locked -  Read HV PHY register
///  @hw: pointer to the HW structure
///  @offset: register offset to be read
///  @data: pointer to the read data
///
///  Reads the PHY register at offset and stores the retrieved information
///  in data.  Assumes semaphore already acquired.
pub fn read_phy_reg_hv_locked(adapter: &mut Adapter, offset: u32, data: &mut u16) -> AdResult {
    __read_phy_reg_hv(adapter, offset, data, true, false)
}

///  e1000_read_phy_reg_page_hv - Read HV PHY register
///  @hw: pointer to the HW structure
///  @offset: register offset to write to
///  @data: data to write at register offset
///
///  Reads the PHY register at offset and stores the retrieved information
///  in data.  Assumes semaphore already acquired and page already set.
pub fn read_phy_reg_page_hv(adapter: &mut Adapter, offset: u32, data: &mut u16) -> AdResult {
    __read_phy_reg_hv(adapter, offset, data, true, true)
}

/// __e1000_write_phy_reg_hv - Write HV PHY register
/// @hw: pointer to the HW structure
/// @offset: register offset to write to
/// @data: data to write at register offset
/// @locked: semaphore has already been acquired or not
///
/// Acquires semaphore, if necessary, then writes the data to PHY register
/// at the offset.  Release any acquired semaphores before exiting.
pub fn __write_phy_reg_hv(
    adapter: &mut Adapter,
    offset: u32,
    data: u16,
    locked: bool,
    page_set: bool,
) -> AdResult {
    e1000_verbose_println!();

    fn release_if_locked(adapter: &mut Adapter, locked: bool) -> AdResult {
        if !locked {
            adapter
                .hw
                .phy
                .ops
                .release
                .ok_or("No function".to_string())
                .and_then(|f| f(adapter))
        } else {
            Ok(())
        }
    };

    let mut data = data;
    let mut page: u16 = BM_PHY_REG_PAGE(offset);
    let reg: u16 = BM_PHY_REG_NUM(offset);
    let phy_addr: u32 = get_phy_addr_for_hv_page(page as u32);
    adapter.hw.phy.addr = phy_addr;

    if !locked {
        try!(adapter.phy_acquire());
    }
    /* Page 800 works differently than the rest so it has its own func */
    if page == BM_WUC_PAGE as u16 {
        let res = access_phy_wakeup_reg_bm(adapter, offset, &mut data, false, page_set);
        return release_if_locked(adapter, locked).and(res);
    }

    if page > 0 && page < HV_INTC_FC_PAGE_START as u16 {
        let res = access_phy_debug_regs_hv(adapter, offset, &mut data, false);
        return release_if_locked(adapter, locked).and(res);
    }

    if !page_set {
        if page == HV_INTC_FC_PAGE_START as u16 {
            page = 0;
        }
        if adapter.hw.phy.phy_type == PhyType::Type_82578 && adapter.hw.phy.revision >= 1 &&
            adapter.hw.phy.addr == 2 &&
            !btst!(reg as u32, MAX_PHY_REG_ADDRESS) && btst!(data, 1 << 11)
        {
            let mut data2: u16 = 0x7EFF;
            let res = access_phy_debug_regs_hv(adapter, (1 << 6) | 0x3, &mut data2, false);
            if res.is_err() {
                return release_if_locked(adapter, locked).and(res);
            }
        }
        if reg > MAX_PHY_MULTI_PAGE_REG as u16 {
            let res = set_page_igp(adapter, page << IGP_PAGE_SHIFT);
            adapter.hw.phy.addr = phy_addr;
            if res.is_err() {
                return release_if_locked(adapter, locked).and(res);
            }
        }
    }
    e1000_verbose_println!(
        "Writing PHY page {} (or 0x{:x} shifted) reg 0x{:x}, data 0x{:x}",
        page,
        page << IGP_PAGE_SHIFT,
        reg, data
    );

    let res = write_phy_reg_mdic(adapter, reg as u32 & MAX_PHY_REG_ADDRESS, data);

    return release_if_locked(adapter, locked).and(res);
}

/// e1000_write_phy_reg_hv - Write HV PHY register
/// @hw: pointer to the HW structure
/// @offset: register offset to write to
/// @data: data to write at register offset
///
/// Acquires semaphore then writes the data to PHY register at the offset.
/// Release the acquired semaphores before exiting.
pub fn write_phy_reg_hv(adapter: &mut Adapter, offset: u32, data: u16) -> AdResult {
    __write_phy_reg_hv(adapter, offset, data, false, false)
}

/// e1000_write_phy_reg_hv_locked - Write HV PHY register
/// @hw: pointer to the HW structure
/// @offset: register offset to write to
/// @data: data to write at register offset
///
/// Writes the data to PHY register at the offset.  Assumes semaphore
/// already acquired.
pub fn write_phy_reg_hv_locked(adapter: &mut Adapter, offset: u32, data: u16) -> AdResult {
    __write_phy_reg_hv(adapter, offset, data, true, false)
}


/// e1000_write_phy_reg_page_hv - Write HV PHY register
/// @hw: pointer to the HW structure
/// @offset: register offset to write to
/// @data: data to write at register offset
///
/// Writes the data to PHY register at the offset.  Assumes semaphore
/// already acquired and page already set.
pub fn write_phy_reg_page_hv(adapter: &mut Adapter, offset: u32, data: u16) -> AdResult {
    __write_phy_reg_hv(adapter, offset, data, true, true)
}

/// __e1000_read_kmrn_reg - Read kumeran register
/// @hw: pointer to the HW structure
/// @offset: register offset to be read
/// @data: pointer to the read data
/// @locked: semaphore has already been acquired or not
///
/// Acquires semaphore, if necessary.  Then reads the PHY register at offset
/// using the kumeran interface.  The information retrieved is stored in data.
/// Release any acquired semaphores before exiting.
pub fn __read_kmrn_reg(adapter: &mut Adapter, offset: u32, data: &mut u16, locked: bool) -> AdResult {
    e1000_verbose_println!();

    let mut kmrnctrlsta: u32;

    if !locked {
        if adapter.hw.phy.ops.acquire.is_none() {
            e1000_phy_println!("No function - return success");
            return Ok(())
        }
        try!(adapter.phy_acquire());
    }

    kmrnctrlsta = ((offset << E1000_KMRNCTRLSTA_OFFSET_SHIFT) & E1000_KMRNCTRLSTA_OFFSET) |
    E1000_KMRNCTRLSTA_REN;
    adapter.write_register(E1000_KMRNCTRLSTA, kmrnctrlsta);
    adapter.write_flush();

    do_usec_delay(2);

    kmrnctrlsta = adapter.read_register(E1000_KMRNCTRLSTA);
    *data = kmrnctrlsta as u16;

    if !locked {
        try!(adapter.phy_release());
    }

    Ok(())
}

/// e1000_read_kmrn_reg_generic -  Read kumeran register
/// @hw: pointer to the HW structure
/// @offset: register offset to be read
/// @data: pointer to the read data
///
/// Acquires semaphore then reads the PHY register at offset using the
/// kumeran interface.  The information retrieved is stored in data.
/// Release the acquired semaphore before exiting.

pub fn read_kmrn_reg_generic(adapter: &mut Adapter, offset: u32, data: &mut u16) -> AdResult {
    __read_kmrn_reg(adapter, offset, data, false)
}

/// e1000_read_kmrn_reg_locked -  Read kumeran register
/// @hw: pointer to the HW structure
/// @offset: register offset to be read
/// @data: pointer to the read data
///
/// Reads the PHY register at offset using the kumeran interface.  The
/// information retrieved is stored in data.
/// Assumes semaphore already acquired.
pub fn read_kmrn_reg_locked(adapter: &mut Adapter, offset: u32, data: &mut u16) -> AdResult {
    __read_kmrn_reg(adapter, offset, data, true)
}

/// __e1000_write_kmrn_reg - Write kumeran register
/// @hw: pointer to the HW structure
/// @offset: register offset to write to
/// @data: data to write at register offset
/// @locked: semaphore has already been acquired or not
///
/// Acquires semaphore, if necessary.  Then write the data to PHY register
/// at the offset using the kumeran interface.  Release any acquired semaphores
/// before exiting.
pub fn __write_kmrn_reg(adapter: &mut Adapter, offset: u32, data: u16, locked: bool) -> AdResult {
    e1000_verbose_println!();

    let mut kmrnctrlsta: u32 = 0;

    if !locked {
        if adapter.hw.phy.ops.acquire.is_none() {
            e1000_phy_println!("No function - return success");
            return Ok(())
        }
        try!(adapter.phy_acquire());
    }

    kmrnctrlsta = ((offset << E1000_KMRNCTRLSTA_OFFSET_SHIFT) & E1000_KMRNCTRLSTA_OFFSET) | data as u32;
    adapter.write_register(E1000_KMRNCTRLSTA, kmrnctrlsta);
    adapter.write_flush();

    do_usec_delay(2);

    if !locked {
        try!(adapter.phy_release());
    }
    Ok(())
}

/// e1000_write_kmrn_reg_generic -  Write kumeran register
/// @hw: pointer to the HW structure
/// @offset: register offset to write to
/// @data: data to write at register offset
///
/// Acquires semaphore then writes the data to the PHY register at the offset
/// using the kumeran interface.  Release the acquired semaphore before exiting.
pub fn write_kmrn_reg_generic(adapter: &mut Adapter, offset: u32, data: u16) -> AdResult {
    __write_kmrn_reg(adapter, offset, data, false)
}

/// e1000_write_kmrn_reg_locked -  Write kumeran register
/// @hw: pointer to the HW structure
/// @offset: register offset to write to
/// @data: data to write at register offset
///
/// Write the data to PHY register at the offset using the kumeran interface.
/// Assumes semaphore already acquired.
pub fn write_kmrn_reg_locked(adapter: &mut Adapter, offset: u32, data: u16) -> AdResult {
    __write_kmrn_reg(adapter, offset, data, true)
}

/// e1000_get_phy_addr_for_hv_page - Get PHY adrress based on page
///  @page: page to be accessed
pub fn get_phy_addr_for_hv_page(page: u32) -> u32 {
    e1000_verbose_println!();

    let mut phy_addr: u32 = 2;
    if page >= HV_INTC_FC_PAGE_START {
        phy_addr = 1;
    }
    phy_addr
}

/// e1000_get_phy_type_from_id - Get PHY type from id
/// @phy_id: phy_id read from the phy
///
/// Returns the phy type from the id.
pub fn get_phy_type_from_id(phy_id: u32) -> PhyType {
    e1000_phy_println!();

    let p = match phy_id {
        M88E1000_I_PHY_ID |
        M88E1000_E_PHY_ID |
        M88E1111_I_PHY_ID |
        M88E1011_I_PHY_ID |
        M88E1543_E_PHY_ID |
        M88E1512_E_PHY_ID |
        I347AT4_E_PHY_ID |
        M88E1112_E_PHY_ID |
        M88E1340M_E_PHY_ID => PhyType::Type_m88,
        IGP01E1000_I_PHY_ID => PhyType::Type_igp_2,
        GG82563_E_PHY_ID => PhyType::Type_gg82563,
        IGP03E1000_E_PHY_ID => PhyType::Type_igp_3,
        IFE_E_PHY_ID |
        IFE_PLUS_E_PHY_ID |
        IFE_C_E_PHY_ID => PhyType::Type_ife,
        BME1000_E_PHY_ID |
        BME1000_E_PHY_ID_R2 => PhyType::Type_bm,
        I82578_E_PHY_ID => PhyType::Type_82578,
        I82577_E_PHY_ID => PhyType::Type_82577,
        I82579_E_PHY_ID => PhyType::Type_82579,
        I217_E_PHY_ID => PhyType::Type_i217,
        I82580_I_PHY_ID => PhyType::Type_82580,
        I210_I_PHY_ID => PhyType::Type_i210,
        _ => PhyType::Type_unknown,
    };
    e1000_phy_println!("Got phy type: {:?}", p);
    p
}

/// e1000_phy_hw_reset_generic - PHY hardware reset
/// @hw: pointer to the HW structure
///
/// Verify the reset block is not blocking us from resetting.  Acquire
/// semaphore (if necessary) and read/set/write the device control reset
/// bit in the PHY.  Wait the appropriate delay time for the device to
/// reset and release the semaphore (if necessary).
pub fn phy_hw_reset_generic(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();

    match adapter.check_reset_block() {
        Ok(true) => return Ok(()),
        Ok(false) => (),
        Err(e) => eprintln!("{:?}", e)
    };

    try!(adapter.phy_acquire());

    adapter.set_register_bit(E1000_CTRL, E1000_CTRL_PHY_RST);
    adapter.write_flush();

    do_usec_delay(adapter.hw.phy.reset_delay_us as usize);

    adapter.clear_register_bit(E1000_CTRL, E1000_CTRL_PHY_RST);
    adapter.write_flush();

    do_usec_delay(150);

    try!(adapter.phy_release());

    adapter
        .hw
        .phy
        .ops
        .get_cfg_done
        .ok_or("No function".to_string())
        .and_then(|f| f(adapter))
}

/// e1000_phy_sw_reset_generic - PHY software reset
/// @hw: pointer to the HW structure
///
/// Does a software reset of the PHY by reading the PHY control register and
/// setting/write the control register reset bit to the PHY.
pub fn phy_sw_reset_generic(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();
    incomplete_return!();
}

/// e1000_get_cfg_done_generic - Generic configuration done
/// @hw: pointer to the HW structure
///
/// Generic function to wait 10 milli-seconds for configuration to complete
/// and return success.
pub fn get_cfg_done_generic(adapter: &mut Adapter) -> AdResult {
    e1000_verbose_println!();
    do_msec_delay(10);
    Ok(())
}

pub fn get_cable_length_m88(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();
    incomplete_return!();
}

pub fn phy_force_speed_duplex_m88(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();
    incomplete_return!();
}

pub fn check_polarity_m88(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();

    let mut data: u16 = 0;

    try!(adapter.phy_read_reg(M88E1000_PHY_SPEC_STATUS, &mut data));
    adapter.hw.phy.cable_polarity = match btst!(data, M88E1000_PSSR_REV_POLARITY) {
        true => RevPolarity::Reversed,
        false => RevPolarity::Normal,
    };

    Ok(())
}

pub fn read_phy_reg_mdic(adapter: &mut Adapter, offset: u32, data: &mut u16) -> AdResult {
    /// Fatal on error

    let mut mdic: u32;

    if offset > MAX_PHY_REG_ADDRESS {
        return Err(format!("PHY address {} is out of range", offset));
    }

    /* Set up Op-code, Phy Address, and register offset in the MDI
     * Control register.  The MAC will take care of interfacing with the
     * PHY to retrieve the desired data.
     */
    mdic = (offset << E1000_MDIC_REG_SHIFT) | (adapter.hw.phy.addr << E1000_MDIC_PHY_SHIFT) |
        E1000_MDIC_OP_READ;

    do_write_register(adapter, E1000_MDIC, mdic);

    /* Poll the ready bit to see if the MDI read completed
     * Increasing the time out as testing showed failures with
     * the lower time out
     */
    for i in 0..(E1000_GEN_POLL_TIMEOUT * 3) {
        do_usec_delay(50);
        mdic = do_read_register(adapter, E1000_MDIC);
        if btst!(mdic, E1000_MDIC_READY) {
            break;
        }
    }

    if !btst!(mdic, E1000_MDIC_READY) {
        return Err("MDI read did not complete".into());
    }

    if btst!(mdic, E1000_MDIC_ERROR) {
        return Err("MDI error".into());
    }

    if ((mdic & E1000_MDIC_REG_MASK) >> E1000_MDIC_REG_SHIFT) != offset {
        return Err(format!(
            "MDI Read offset error - requested {}, returned {}",
            offset,
            (mdic & E1000_MDIC_REG_MASK) >> E1000_MDIC_REG_SHIFT
        ));
    }
    *data = mdic as u16;

    /* Allow some time after each MDIC transaction to avoid
     * reading duplicate data in the next MDIC transaction.
     */
    if adapter.hw.mac.mac_type == MacType::Mac_pch2lan {
        do_usec_delay(100);
    }

    Ok(())
}

pub fn has_link_generic(
    adapter: &mut Adapter,
    iterations: u32,
    usec_interval: u32,
    success: &mut bool,
) -> AdResult {
    e1000_phy_println!();

    let mut i = 0;
    let mut phy_status: u16 = 0;

    let mut res = Ok(());
    if let Some(read_reg) = adapter.hw.phy.ops.read_reg {

        while i < iterations {
            /* Some PHYs require the PHY_STATUS register to be read
             * twice due to the link bit being sticky.  No harm doing
             * it across the board.
             */
            if read_reg(adapter, PHY_STATUS, &mut phy_status).is_err() {
                /* If the first read fails, another entity may have
                 * ownership of the resources, wait and try again to
                 * see if they have relinquished the resources yet.
                 */
                if usec_interval >= 1000 {
                    do_msec_delay(usec_interval as usize / 1000);
                } else {
                    do_usec_delay(usec_interval as usize);
                }
            }
            res = read_reg(adapter, PHY_STATUS, &mut phy_status);
            if res.is_err() {
                break;
            }
            if btst!(phy_status, MII_SR_LINK_STATUS) {
                break;
            }
            if usec_interval >= 1000 {
                do_msec_delay(usec_interval as usize / 1000);
            } else {
                do_usec_delay(usec_interval as usize);
            }
            i += 1;
        }
        *success = i < iterations;
    } else {
        e1000_phy_println!("read_reg function not set");
    }

    res
}

pub fn check_downshift_generic(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();

    let mut phy_data: u16 = 0;
    let mut offset: u16 = 0;
    let mut mask: u16 = 0;

    match adapter.hw.phy.phy_type {
        PhyType::Type_i210 |
        PhyType::Type_m88 |
        PhyType::Type_gg82563 |
        PhyType::Type_bm |
        PhyType::Type_82578 => {
            offset = M88E1000_PHY_SPEC_STATUS as u16;
            mask = M88E1000_PSSR_DOWNSHIFT as u16;
        }
        PhyType::Type_igp |
        PhyType::Type_igp_2 |
        PhyType::Type_igp_3 => {
            incomplete_return!();
        }
        _ => {
            adapter.hw.phy.speed_downgraded = false;
            return Ok(());
        }
    }

    try!(adapter.phy_read_reg(offset as u32, &mut phy_data));
    adapter.hw.phy.speed_downgraded = btst!(phy_data, mask);

    Ok(())
}

/// e1000_set_page_igp - Set page as on IGP-like PHY(s)
/// @hw: pointer to the HW structure
/// @page: page to set (shifted left when necessary)
///
/// Sets PHY page required for PHY register access.  Assumes semaphore is
/// already acquired.  Note, this function sets phy.addr to 1 so the caller
/// must set it appropriately (if necessary) after this function returns.
pub fn set_page_igp(adapter: &mut Adapter, page: u16) -> AdResult {
    e1000_verbose_println!();

    e1000_verbose_println!("Setting page 0x{:x}", page);
    adapter.hw.phy.addr = 1;
    write_phy_reg_mdic(adapter, IGP01E1000_PHY_PAGE_SELECT, page)
}


/// e1000_access_phy_debug_regs_hv - Read HV PHY vendor specific high registers
/// @hw: pointer to the HW structure
/// @offset: register offset to be read or written
/// @data: pointer to the data to be read or written
/// @read: determines if operation is read or write
///
/// Reads the PHY register at offset and stores the retreived information
/// in data.  Assumes semaphore already acquired.  Note that the procedure
/// to access these regs uses the address port and data port to read/write.
/// These accesses done with PHY address 2 and without using pages.
pub fn access_phy_debug_regs_hv(
    adapter: &mut Adapter,
    offset: u32,
    data: &mut u16,
    read: bool,
) -> AdResult {
    e1000_phy_println!();
    incomplete_return!();
}

/// e1000_access_phy_wakeup_reg_bm - Read/write BM PHY wakeup register
/// @hw: pointer to the HW structure
/// @offset: register offset to be read or written
/// @data: pointer to the data to read or write
/// @read: determines if operation is read or write
/// @page_set: BM_WUC_PAGE already set and access enabled
///
/// Read the PHY register at offset and store the retrieved information in
/// data, or write data to PHY register at offset.  Note the procedure to
/// access the PHY wakeup registers is different than reading the other PHY
/// registers. It works as such:
/// 1) Set 769.17.2 (page 769, register 17, bit 2) = 1
/// 2) Set page to 800 for host (801 if we were manageability)
/// 3) Write the address using the address opcode (0x11)
/// 4) Read or write the data using the data opcode (0x12)
/// 5) Restore 769.17.2 to its original value
///
/// Steps 1 and 2 are done by e1000_enable_phy_wakeup_reg_access_bm() and
/// step 5 is done by e1000_disable_phy_wakeup_reg_access_bm().
///
/// Assumes semaphore is already acquired.  When page_set==TRUE, assumes
/// the PHY page is set to BM_WUC_PAGE (i.e. a function in the call stack
/// is responsible for calls to e1000_[enable|disable]_phy_wakeup_reg_bm()).
pub fn access_phy_wakeup_reg_bm(
    adapter: &mut Adapter,
    offset: u32,
    data: &mut u16,
    read: bool,
    page_set: bool,
) -> AdResult {
    e1000_verbose_println!();

    let mut reg: u16 = BM_PHY_REG_NUM(offset);
    let mut page: u16 = BM_PHY_REG_PAGE(offset);
    let mut phy_reg: u16 = 0;

    /* Gig must be disabled for MDIO accesses to Host Wakeup reg page */
    if adapter.hw.mac.mac_type == MacType::Mac_pchlan {
        incomplete_return!();
    }

    if !page_set {
        /* Enable access to PHY wakeup registers */
        try!(enable_phy_wakeup_reg_access_bm(adapter, &mut phy_reg));
    }

    /* Write the Wakeup register page offset value using opcode 0x11 */
    try!(write_phy_reg_mdic(adapter, BM_WUC_ADDRESS_OPCODE, reg));

    if read {
        /* Read the Wakeup register page value using opcode 0x12 */
        try!(read_phy_reg_mdic(adapter, BM_WUC_DATA_OPCODE, data));
    } else {
        /* Write the Wakeup register page value using opcode 0x12 */
        try!(write_phy_reg_mdic(adapter, BM_WUC_DATA_OPCODE, *data));
    }

    if !page_set {
        try!(disable_phy_wakeup_reg_access_bm(adapter, &mut phy_reg));
    }

    Ok(())
}


/// e1000_check_polarity_82577 - Checks the polarity.
/// @hw: pointer to the HW structure
///
/// Success returns 0, Failure returns -E1000_ERR_PHY (-2)
///
/// Polarity is determined based on the PHY specific status register.
pub fn check_polarity_82577(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();
    incomplete_return!();
}

/// e1000_phy_force_speed_duplex_82577 - Force speed/duplex for I82577 PHY
/// @hw: pointer to the HW structure
///
/// Calls the PHY setup function to force speed and duplex.
pub fn phy_force_speed_duplex_82577(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();
    incomplete_return!();
}

/// e1000_get_cable_length_82577 - Determine cable length for 82577 PHY
/// @hw: pointer to the HW structure
///
/// Reads the diagnostic status register and verifies result is valid before
/// placing it in the phy_cable_length field.
pub fn get_cable_length_82577(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();
    incomplete_return!();
}

/// e1000_get_phy_info_82577 - Retrieve I82577 PHY information
/// @hw: pointer to the HW structure
///
/// Read PHY status to determine if link is up.  If link is up, then
/// set/determine 10base-T extended distance and polarity correction.  Read
/// PHY port status to determine MDI/MDIx and speed.  Based on the speed,
/// determine on the cable length, local and remote receiver.
pub fn get_phy_info_82577(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();

    let mut data: u16 = 0;
    let mut link: bool = false;

    try!(has_link_generic(adapter, 1, 0, &mut link));
    if !link {
        return Err("Phy info is only valid if link is up".to_string());
    }

    adapter.hw.phy.polarity_correction = true;
    try!(check_polarity_82577(adapter));
    try!(adapter.phy_read_reg(I82577_PHY_STATUS_2, &mut data));

    adapter.hw.phy.is_mdix = btst!(data, I82577_PHY_STATUS2_MDIX);

    if data & I82577_PHY_STATUS2_SPEED_MASK == I82577_PHY_STATUS2_SPEED_1000MBPS {
        try!(adapter.hw.phy.ops.get_cable_length.ok_or("No function".to_string()).and_then(|f|{
            f(adapter)
        }));
        try!(adapter.phy_read_reg(PHY_1000T_STATUS, &mut data));
        adapter.hw.phy.local_rx = match btst!(data, SR_1000T_LOCAL_RX_STATUS) {
            true => GbRxStatus::Ok,
            false => GbRxStatus::NotOk
        };
        adapter.hw.phy.remote_rx = match btst!(data, SR_1000T_REMOTE_RX_STATUS) {
            true => GbRxStatus::Ok,
            false => GbRxStatus::NotOk
        };
    } else {
        adapter.hw.phy.cable_length = E1000_CABLE_LENGTH_UNDEFINED;
        adapter.hw.phy.local_rx = GbRxStatus::Undefined;
        adapter.hw.phy.remote_rx = GbRxStatus::Undefined;
    }

    Ok(())
}

/// e1000_copper_link_setup_82577 - Setup 82577 PHY for copper link
/// @hw: pointer to the HW structure
///
/// Sets up Carrier-sense on Transmit and downshift values.
pub fn copper_link_setup_82577(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();

    let mut phy_data: u16 = 0;

    if adapter.hw.phy.phy_type == PhyType::Type_82580 {
        try!(adapter.phy_reset());
    }

    /* Enable CRS on Tx. This must be set for half-duplex operation. */
    try!(adapter.phy_read_reg(I82577_CFG_REG, &mut phy_data));

    phy_data |= I82577_CFG_ASSERT_CRS_ON_TX;

    /* Enable downshift */
    phy_data |= I82577_CFG_ENABLE_DOWNSHIFT;
    try!(adapter.phy_write_reg(I82577_CFG_REG, phy_data));

    /* Set MDI/MDIX mode */
    try!(adapter.phy_read_reg(I82577_PHY_CTRL_2, &mut phy_data));

    phy_data &= !(I82577_PHY_CTRL2_MDIX_CFG_MASK);

    /* Options:
     *   0 - Auto (default)
     *   1 - MDI mode
     *   2 - MDI-X mode
     */
    match adapter.hw.phy.mdix {
        1 => (),
        2 => phy_data |= I82577_PHY_CTRL2_MANUAL_MDIX,
        _ => phy_data |= I82577_PHY_CTRL2_AUTO_MDI_MDIX,
    }

    try!(adapter.phy_write_reg(I82577_PHY_CTRL_2, phy_data));

    set_master_slave_mode(adapter)
}

/// e1000_set_master_slave_mode - Setup PHY for Master/slave mode
/// @hw: pointer to the HW structure
///
/// Sets up Master/slave mode
pub fn set_master_slave_mode(adapter: &mut Adapter) -> AdResult {

    let mut phy_data: u16 = 0;

    /* Resolve Master/Slave mode */
    try!(adapter.phy_read_reg(PHY_1000T_CTRL, &mut phy_data));

    /* load defaults for future use */
    adapter.hw.phy.original_ms_type = match btst!(phy_data, CR_1000T_MS_ENABLE) {
        true => match btst!(phy_data, CR_1000T_MS_VALUE) {
            true => MsType::ForceMaster,
            false => MsType::ForceSlave
        }
        false => MsType::Auto
    };

    match adapter.hw.phy.ms_type {
        MsType::ForceMaster => phy_data |= CR_1000T_MS_ENABLE | CR_1000T_MS_VALUE,
        MsType::ForceSlave => {
            phy_data |= CR_1000T_MS_ENABLE;
            phy_data &= !(CR_1000T_MS_VALUE);
        },
        MsType::Auto => phy_data &= !(CR_1000T_MS_ENABLE),
        _ => ()
    }
    adapter.phy_write_reg(PHY_1000T_CTRL, phy_data)
}

/// e1000_enable_phy_wakeup_reg_access_bm - enable access to BM wakeup registers
/// @hw: pointer to the HW structure
/// @phy_reg: pointer to store original contents of BM_WUC_ENABLE_REG
///
/// Assumes semaphore already acquired and phy_reg points to a valid memory
/// address to store contents of the BM_WUC_ENABLE_REG register.
pub fn enable_phy_wakeup_reg_access_bm(adapter: &mut Adapter, phy_reg: &mut u16) -> AdResult {
    e1000_println!();

    let mut temp: u16;

    /* All page select, port ctrl and wakeup registers use phy address 1 */
    adapter.hw.phy.addr = 1;

    /* Select Port Control Registers page */
    try!(set_page_igp(adapter, (BM_PORT_CTRL_PAGE << IGP_PAGE_SHIFT) as u16));
    try!(read_phy_reg_mdic(adapter, BM_WUC_ENABLE_REG, phy_reg));

    /* Enable both PHY wakeup mode and Wakeup register page writes.
     * Prevent a power state change by disabling ME and Host PHY wakeup.
     */
    temp = *phy_reg;
    temp |= BM_WUC_ENABLE_BIT as u16;
    temp &= !((BM_WUC_ME_WU_BIT | BM_WUC_HOST_WU_BIT) as u16);
    try!(write_phy_reg_mdic(adapter, BM_WUC_ENABLE_REG, temp));

    /* Select Host Wakeup Registers page - caller now able to write
     * registers on the Wakeup registers page
     */
    set_page_igp(adapter, (BM_WUC_PAGE << IGP_PAGE_SHIFT) as u16)
}

/// e1000_disable_phy_wakeup_reg_access_bm - disable access to BM wakeup regs
/// @hw: pointer to the HW structure
/// @phy_reg: pointer to original contents of BM_WUC_ENABLE_REG
///
/// Restore BM_WUC_ENABLE_REG to its original value.
///
/// Assumes semaphore already acquired and *phy_reg is the contents of the
/// BM_WUC_ENABLE_REG before register(s) on BM_WUC_PAGE were accessed by
/// caller.
pub fn disable_phy_wakeup_reg_access_bm(adapter: &mut Adapter, phy_reg: &mut u16) -> AdResult {
    e1000_println!();

    /* Select Port Control Registers page */
    try!(set_page_igp(adapter, (BM_PORT_CTRL_PAGE << IGP_PAGE_SHIFT) as u16));

    /* Restore 769.17 to its original value */
    write_phy_reg_mdic(adapter, BM_WUC_ENABLE_REG, *phy_reg)
}
