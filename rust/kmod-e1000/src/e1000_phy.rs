
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

    // struct e1000_phy_info *phy = &hw->phy;
    // s32 ret_val;
    // u16 phy_ctrl;
    let mut phy_ctrl: u16 = 0;

    let read_reg = try!(adapter.hw.phy.ops.read_reg.ok_or("No function".to_string()));
    let write_reg = try!(adapter.hw.phy.ops.write_reg.ok_or(
        "No function".to_string(),
    ));

    // DEBUGFUNC("e1000_copper_link_autoneg");

    /* Perform some bounds checking on the autoneg advertisement
     * parameter.
     */
    // phy->autoneg_advertised &= phy->autoneg_mask;
    adapter.hw.phy.autoneg_advertised &= adapter.hw.phy.autoneg_mask;

    /* If autoneg_advertised is zero, we assume it was not defaulted
     * by the calling code so we set to advertise full capability.
    //  */
    // if (!phy->autoneg_advertised)
    //     phy->autoneg_advertised = phy->autoneg_mask;
    if adapter.hw.phy.autoneg_advertised == 0 {
        adapter.hw.phy.autoneg_advertised = adapter.hw.phy.autoneg_mask;
    }

    // DEBUGOUT("Reconfiguring auto-neg advertisement params\n");
    // ret_val = e1000_phy_setup_autoneg(hw);
    // if (ret_val) {
    //     DEBUGOUT("Error Setting up Auto-Negotiation\n");
    //     return ret_val;
    // }
    // DEBUGOUT("Restarting Auto-Neg\n");
    e1000_phy_println!("Reconfiguring auto-neg advertisement params");
    try!(setup_autoneg(adapter));
    e1000_phy_println!("Restarting Auto-Neg");

    /* Restart auto-negotiation by setting the Auto Neg Enable bit and
     * the Auto Neg Restart bit in the PHY control register.
     */
    // ret_val = phy->ops.read_reg(hw, PHY_CONTROL, &phy_ctrl);
    // if (ret_val)
    //     return ret_val;
    try!(read_reg(adapter, PHY_CONTROL, &mut phy_ctrl));

    // phy_ctrl |= (MII_CR_AUTO_NEG_EN | MII_CR_RESTART_AUTO_NEG);
    // ret_val = phy->ops.write_reg(hw, PHY_CONTROL, phy_ctrl);
    // if (ret_val)
    //     return ret_val;
    phy_ctrl |= MII_CR_AUTO_NEG_EN | MII_CR_RESTART_AUTO_NEG;
    try!(write_reg(adapter, PHY_CONTROL, phy_ctrl));

    /* Does the user want to wait for Auto-Neg to complete here, or
     * check at a later time (for example, callback routine).
     */
    // if (phy->autoneg_wait_to_complete) {
    //     ret_val = e1000_wait_autoneg(hw);
    //     if (ret_val) {
    //         DEBUGOUT("Error while waiting for autoneg to complete\n");
    //         return ret_val;
    //     }
    // }
    if adapter.hw.phy.autoneg_wait_to_complete {
        try!(wait_autoneg(adapter));
    }
    // hw->mac.get_link_status = TRUE;
    adapter.hw.mac.get_link_status = true;

    // return ret_val;
    Ok(())
}

pub fn wait_autoneg(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();

    // s32 ret_val = E1000_SUCCESS;
    // u16 i, phy_status;
    let mut phy_status: u16 = 0;
    // DEBUGFUNC("e1000_wait_autoneg");

    // if (!hw->phy.ops.read_reg)
    //     return E1000_SUCCESS;
    /* Break after autoneg completes or PHY_AUTO_NEG_LIMIT expires. */
    // for (i = PHY_AUTO_NEG_LIMIT; i > 0; i--) {
    //     ret_val = hw->phy.ops.read_reg(hw, PHY_STATUS, &phy_status);
    //     if (ret_val)
    //         break;
    //     ret_val = hw->phy.ops.read_reg(hw, PHY_STATUS, &phy_status);
    //     if (ret_val)
    //         break;
    //     if (phy_status & MII_SR_AUTONEG_COMPLETE)
    //         break;
    //     msec_delay(100);
    // }

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
    // return ret_val;
    Ok(())
}

pub fn setup_autoneg(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();

    // struct e1000_phy_info *phy = &hw->phy;
    // s32 ret_val;
    // u16 mii_autoneg_adv_reg;
    // u16 mii_1000t_ctrl_reg = 0;
    let mut mii_autoneg_adv_reg: u16 = 0;
    let mut mii_1000t_ctrl_reg: u16 = 0;

    // DEBUGFUNC("e1000_phy_setup_autoneg");

    // phy->autoneg_advertised &= phy->autoneg_mask;
    adapter.hw.phy.autoneg_advertised &= adapter.hw.phy.autoneg_mask;

    let read_reg = try!(adapter.hw.phy.ops.read_reg.ok_or("No function".to_string()));
    let write_reg = try!(adapter.hw.phy.ops.write_reg.ok_or(
        "No function".to_string(),
    ));

    // /* Read the MII Auto-Neg Advertisement Register (Address 4). */
    // ret_val = phy->ops.read_reg(hw, PHY_AUTONEG_ADV, &mii_autoneg_adv_reg);
    // if (ret_val)
    //     return ret_val;
    try!(read_reg(adapter, PHY_AUTONEG_ADV, &mut mii_autoneg_adv_reg));

    // if (phy->autoneg_mask & ADVERTISE_1000_FULL) {
    //     /* Read the MII 1000Base-T Control Register (Address 9). */
    //     ret_val = phy->ops.read_reg(hw, PHY_1000T_CTRL,
    //     			    &mii_1000t_ctrl_reg);
    //     if (ret_val)
    //         return ret_val;
    // }
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
    // mii_autoneg_adv_reg &= ~(NWAY_AR_100TX_FD_CAPS |
    //     		     NWAY_AR_100TX_HD_CAPS |
    //     		     NWAY_AR_10T_FD_CAPS   |
    //     		     NWAY_AR_10T_HD_CAPS);
    // mii_1000t_ctrl_reg &= ~(CR_1000T_HD_CAPS | CR_1000T_FD_CAPS);

    // DEBUGOUT1("autoneg_advertised %x\n", phy->autoneg_advertised);

    mii_autoneg_adv_reg &= !(NWAY_AR_100TX_FD_CAPS | NWAY_AR_100TX_HD_CAPS | NWAY_AR_10T_FD_CAPS |
                                 NWAY_AR_10T_HD_CAPS);
    mii_1000t_ctrl_reg &= !(CR_1000T_HD_CAPS | CR_1000T_FD_CAPS);

    /* Do we want to advertise 10 Mb Half Duplex? */
    // if (phy->autoneg_advertised & ADVERTISE_10_HALF) {
    //     DEBUGOUT("Advertise 10mb Half duplex\n");
    //     mii_autoneg_adv_reg |= NWAY_AR_10T_HD_CAPS;
    // }
    if adapter.hw.phy.autoneg_advertised & ADVERTISE_10_HALF != 0 {
        e1000_phy_println!("Advertise 10mb Half duplex");
        mii_autoneg_adv_reg |= NWAY_AR_10T_HD_CAPS;
    }

    /* Do we want to advertise 10 Mb Full Duplex? */
    // if (phy->autoneg_advertised & ADVERTISE_10_FULL) {
    //     DEBUGOUT("Advertise 10mb Full duplex\n");
    //     mii_autoneg_adv_reg |= NWAY_AR_10T_FD_CAPS;
    // }
    if adapter.hw.phy.autoneg_advertised & ADVERTISE_10_FULL != 0 {
        e1000_phy_println!("Advertise 10mb Full duplex");
        mii_autoneg_adv_reg |= NWAY_AR_10T_FD_CAPS;
    }

    /* Do we want to advertise 100 Mb Half Duplex? */
    // if (phy->autoneg_advertised & ADVERTISE_100_HALF) {
    //     DEBUGOUT("Advertise 100mb Half duplex\n");
    //     mii_autoneg_adv_reg |= NWAY_AR_100TX_HD_CAPS;
    // }
    if adapter.hw.phy.autoneg_advertised & ADVERTISE_100_HALF != 0 {
        e1000_phy_println!("Advertise 100mb Half duplex");
        mii_autoneg_adv_reg |= NWAY_AR_100TX_HD_CAPS;
    }

    /* Do we want to advertise 100 Mb Full Duplex? */
    // if (phy->autoneg_advertised & ADVERTISE_100_FULL) {
    //     DEBUGOUT("Advertise 100mb Full duplex\n");
    //     mii_autoneg_adv_reg |= NWAY_AR_100TX_FD_CAPS;
    // }
    if adapter.hw.phy.autoneg_advertised & ADVERTISE_100_FULL != 0 {
        e1000_phy_println!("Advertise 100mb Full duplex");
        mii_autoneg_adv_reg |= NWAY_AR_100TX_FD_CAPS;
    }

    /* We do not allow the Phy to advertise 1000 Mb Half Duplex */
    // if (phy->autoneg_advertised & ADVERTISE_1000_HALF)
    //     DEBUGOUT("Advertise 1000mb Half duplex request denied!\n");
    if adapter.hw.phy.autoneg_advertised & ADVERTISE_1000_HALF != 0 {
        e1000_phy_println!("Advertise 1000mb half duplex request denied");
    }

    /* Do we want to advertise 1000 Mb Full Duplex? */
    // if (phy->autoneg_advertised & ADVERTISE_1000_FULL) {
    //     DEBUGOUT("Advertise 1000mb Full duplex\n");
    //     mii_1000t_ctrl_reg |= CR_1000T_FD_CAPS;
    // }
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
    // switch (hw->fc.current_mode) {
    //     case e1000_fc_none:
    //     /* Flow control (Rx & Tx) is completely disabled by a
    //      * software over-ride.
    //      */
    //     mii_autoneg_adv_reg &= ~(NWAY_AR_ASM_DIR | NWAY_AR_PAUSE);
    //     break;
    //     case e1000_fc_rx_pause:
    //     /* Rx Flow control is enabled, and Tx Flow control is
    //      * disabled, by a software over-ride.
    //      *
    //      * Since there really isn't a way to advertise that we are
    //      * capable of Rx Pause ONLY, we will advertise that we
    //      * support both symmetric and asymmetric Rx PAUSE.  Later
    //      * (in e1000_config_fc_after_link_up) we will disable the
    //      * hw's ability to send PAUSE frames.
    //      */
    //     mii_autoneg_adv_reg |= (NWAY_AR_ASM_DIR | NWAY_AR_PAUSE);
    //     break;
    //     case e1000_fc_tx_pause:
    //     /* Tx Flow control is enabled, and Rx Flow control is
    //      * disabled, by a software over-ride.
    //      */
    //     mii_autoneg_adv_reg |= NWAY_AR_ASM_DIR;
    //     mii_autoneg_adv_reg &= ~NWAY_AR_PAUSE;
    //     break;
    //     case e1000_fc_full:
    //     /* Flow control (both Rx and Tx) is enabled by a software
    //      * over-ride.
    //      */
    //     mii_autoneg_adv_reg |= (NWAY_AR_ASM_DIR | NWAY_AR_PAUSE);
    //     break;
    //     default:
    //     DEBUGOUT("Flow control param set incorrectly\n");
    //     return -E1000_ERR_CONFIG;
    // }
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
    // ret_val = phy->ops.write_reg(hw, PHY_AUTONEG_ADV, mii_autoneg_adv_reg);
    // if (ret_val)
    //     return ret_val;
    try!(write_reg(adapter, PHY_AUTONEG_ADV, mii_autoneg_adv_reg));

    // DEBUGOUT1("Auto-Neg Advertising %x\n", mii_autoneg_adv_reg);
    e1000_phy_println!("Auto-Neg Advertising 0x{:x}", mii_autoneg_adv_reg);

    // if (phy->autoneg_mask & ADVERTISE_1000_FULL)
    //     ret_val = phy->ops.write_reg(hw, PHY_1000T_CTRL,
    //     			     mii_1000t_ctrl_reg);
    if adapter.hw.phy.autoneg_mask & ADVERTISE_1000_FULL != 0 {
        e1000_phy_println!("Auto-Neg Advertising 1000 Full");
        try!(write_reg(adapter, PHY_1000T_CTRL, mii_1000t_ctrl_reg));
    }
    // return ret_val;
    // incomplete!();
    Ok(())
}

pub fn setup_copper_link_generic(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();

    // s32 ret_val;
    // bool link;

    // DEBUGFUNC("e1000_setup_copper_link_generic");

    // if (hw->mac.autoneg) {
    //     /* Setup autoneg and flow control advertisement and perform
    //      * autonegotiation.
    //      */
    //     ret_val = e1000_copper_link_autoneg(hw);
    //     if (ret_val)
    //         return ret_val;
    // } else {
    //     /* PHY will be set to 10H, 10F, 100H or 100F
    //      * depending on user settings.
    //      */
    //     DEBUGOUT("Forcing Speed and Duplex\n");
    //     ret_val = hw->phy.ops.force_speed_duplex(hw);
    //     if (ret_val) {
    //         DEBUGOUT("Error Forcing Speed and Duplex\n");
    //         return ret_val;
    //     }
    // }
    if adapter.hw.mac.autoneg {
        try!(self::copper_link_autoneg(adapter));
    } else {
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
    // ret_val = e1000_phy_has_link_generic(hw, COPPER_LINK_UP_LIMIT, 10,
    //     				 &link);
    // if (ret_val)
    //     return ret_val;
    let mut link = false;
    try!(has_link_generic(
        adapter,
        COPPER_LINK_UP_LIMIT,
        10,
        &mut link,
    ));

    // if (link) {
    //     DEBUGOUT("Valid link established!!!\n");
    //     hw->mac.ops.config_collision_dist(hw);
    //     ret_val = e1000_config_fc_after_link_up_generic(hw);
    // } else {
    //     DEBUGOUT("Unable to establish link!!!\n");
    // }

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

    // struct e1000_phy_info *phy = &hw->phy;
    // s32 ret_val = E1000_SUCCESS;
    // u16 phy_id;
    // u16 retry_count = 0;

    let mut retry_count = 0;
    let mut phy_id: u16 = 0;

    // DEBUGFUNC("e1000_get_phy_id");


    // if (!phy->ops.read_reg)
    // 	return E1000_SUCCESS;

    // while (retry_count < 2) {
    // 	ret_val = phy->ops.read_reg(hw, PHY_ID1, &phy_id);
    // 	if (ret_val)
    // 		return ret_val;

    // 	phy->id = (u32)(phy_id << 16);
    // 	usec_delay(20);
    // 	ret_val = phy->ops.read_reg(hw, PHY_ID2, &phy_id);
    // 	if (ret_val)
    // 		return ret_val;

    // 	phy->id |= (u32)(phy_id & PHY_REVISION_MASK);
    // 	phy->revision = (u32)(phy_id & ~PHY_REVISION_MASK);

    // 	if (phy->id != 0 && phy->id != PHY_REVISION_MASK)
    // 		return E1000_SUCCESS;

    // 	retry_count++;
    // }

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
    // This function is complete
    Ok(())
}

pub fn power_up_phy_copper(adapter: &mut Adapter) {
    e1000_phy_println!();
    incomplete!();
}

pub fn get_phy_info_m88(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();

    // struct e1000_phy_info *phy = &hw->phy;
    // s32  ret_val;
    // u16 phy_data;
    // bool link;
    let mut phy_data: u16 = 0;
    let mut link: bool = false;
    let read_reg = try!(adapter.hw.phy.ops.read_reg.ok_or("No function".to_string()));

    // DEBUGFUNC("e1000_get_phy_info_m88");

    // if (phy->media_type != e1000_media_type_copper) {
    //     DEBUGOUT("Phy info is only valid for copper media\n");
    //     return -E1000_ERR_CONFIG;
    // }
    if adapter.hw.phy.media_type != MediaType::Copper {
        return Err("Phy info is only valid for copper media".to_string());
    }
    // ret_val = e1000_phy_has_link_generic(hw, 1, 0, &link);
    // if (ret_val)
    //     return ret_val;
    try!(self::has_link_generic(adapter, 1, 0, &mut link));

    // if (!link) {
    //     DEBUGOUT("Phy info is only valid if link is up\n");
    //     return -E1000_ERR_CONFIG;
    // }
    if !link {
        return Err("Phy info is only valid if link is up".to_string());
    }

    // ret_val = phy->ops.read_reg(hw, M88E1000_PHY_SPEC_CTRL, &phy_data);
    // if (ret_val)
    //     return ret_val;
    try!(read_reg(adapter, M88E1000_PHY_SPEC_CTRL, &mut phy_data));

    // phy->polarity_correction = !!(phy_data &
    //     			  M88E1000_PSCR_POLARITY_REVERSAL);
    adapter.hw.phy.polarity_correction = phy_data & M88E1000_PSCR_POLARITY_REVERSAL != 0;

    // ret_val = e1000_check_polarity_m88(hw);
    // if (ret_val)
    //     return ret_val;
    try!(self::check_polarity_m88(adapter));

    // ret_val = phy->ops.read_reg(hw, M88E1000_PHY_SPEC_STATUS, &phy_data);
    // if (ret_val)
    //     return ret_val;
    try!(read_reg(adapter, M88E1000_PHY_SPEC_STATUS, &mut phy_data));

    // phy->is_mdix = !!(phy_data & M88E1000_PSSR_MDIX);
    adapter.hw.phy.is_mdix = phy_data & M88E1000_PSSR_MDIX != 0;

    // if ((phy_data & M88E1000_PSSR_SPEED) == M88E1000_PSSR_1000MBS) {
    //     ret_val = hw->phy.ops.get_cable_length(hw);
    //     if (ret_val)
    //         return ret_val;

    //     ret_val = phy->ops.read_reg(hw, PHY_1000T_STATUS, &phy_data);
    //     if (ret_val)
    //         return ret_val;

    //     phy->local_rx = (phy_data & SR_1000T_LOCAL_RX_STATUS)
    //         ? e1000_1000t_rx_status_ok
    //         : e1000_1000t_rx_status_not_ok;

    //     phy->remote_rx = (phy_data & SR_1000T_REMOTE_RX_STATUS)
    //         ? e1000_1000t_rx_status_ok
    //         : e1000_1000t_rx_status_not_ok;
    // } else {
    //     /* Set values to "undefined" */
    //     phy->cable_length = E1000_CABLE_LENGTH_UNDEFINED;
    //     phy->local_rx = e1000_1000t_rx_status_undefined;
    //     phy->remote_rx = e1000_1000t_rx_status_undefined;
    // }

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


    // return ret_val;
    Ok(())
}

pub fn write_phy_reg_m88(adapter: &mut Adapter, offset: u32, data: u16) -> AdResult {
    e1000_verbose_println!();

    // if (!hw->phy.ops.acquire)
    //     return E1000_SUCCESS;
    // ret_val = hw->phy.ops.acquire(hw);
    // if (ret_val)
    //     return ret_val;
    // ret_val = e1000_write_phy_reg_mdic(hw, MAX_PHY_REG_ADDRESS & offset,
    //     			       data);
    // hw->phy.ops.release(hw);
    // return ret_val;

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
    // s32 ret_val;
    // DEBUGFUNC("e1000_read_phy_reg_m88");
    // if (!hw->phy.ops.acquire)
    // 	return E1000_SUCCESS;
    // ret_val = hw->phy.ops.acquire(hw);
    // if (ret_val)
    // 	return ret_val;
    // ret_val = e1000_read_phy_reg_mdic(hw, MAX_PHY_REG_ADDRESS & offset,
    // 				  data);
    // hw->phy.ops.release(hw);
    // return ret_val;
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
    // struct e1000_phy_info *phy = &hw->phy;
    // u32 i, mdic = 0;

    // DEBUGFUNC("e1000_write_phy_reg_mdic");

    // if (offset > MAX_PHY_REG_ADDRESS) {
    //     DEBUGOUT1("PHY Address %d is out of range\n", offset);
    //     return -E1000_ERR_PARAM;
    // }
    if offset > MAX_PHY_REG_ADDRESS {
        return Err(format!("PHY address {} is out of range", offset));
    }
    /* Set up Op-code, Phy Address, and register offset in the MDI
     * Control register.  The MAC will take care of interfacing with the
     * PHY to retrieve the desired data.
     */
    // mdic = (((u32)data) |
    //         (offset << E1000_MDIC_REG_SHIFT) |
    //         (phy->addr << E1000_MDIC_PHY_SHIFT) |
    //     	        (E1000_MDIC_OP_WRITE));

    let mut mdic: u32 = (data as u32) | (offset << E1000_MDIC_REG_SHIFT) |
        (adapter.hw.phy.addr << E1000_MDIC_PHY_SHIFT) | E1000_MDIC_OP_WRITE;

    // E1000_WRITE_REG(hw, E1000_MDIC, mdic);
    do_write_register(adapter, E1000_MDIC, mdic);

    /* Poll the ready bit to see if the MDI read completed
     * Increasing the time out as testing showed failures with
     * the lower time out
     */
    // for (i = 0; i < (E1000_GEN_POLL_TIMEOUT * 3); i++) {
    //     usec_delay_irq(50);
    //     mdic = E1000_READ_REG(hw, E1000_MDIC);
    //     if (mdic & E1000_MDIC_READY)
    //         break;
    // }
    for i in 0..(E1000_GEN_POLL_TIMEOUT * 3) {
        do_usec_delay(50);
        mdic = do_read_register(adapter, E1000_MDIC);
        if mdic & E1000_MDIC_READY != 0 {
            break;
        }
    }

    // if (!(mdic & E1000_MDIC_READY)) {
    //     DEBUGOUT("MDI Write did not complete\n");
    //     return -E1000_ERR_PHY;
    // }
    if mdic & E1000_MDIC_READY == 0 {
        return Err("MDI write did not complete".to_string());
    }

    // if (mdic & E1000_MDIC_ERROR) {
    //     DEBUGOUT("MDI Error\n");
    //     return -E1000_ERR_PHY;
    // }
    if mdic & E1000_MDIC_ERROR != 0 {
        return Err("MDI error".to_string());
    }

    // if (((mdic & E1000_MDIC_REG_MASK) >> E1000_MDIC_REG_SHIFT) != offset) {
    //     DEBUGOUT2("MDI Write offset error - requested %d, returned %d\n",
    //     	  offset,
    //     	  (mdic & E1000_MDIC_REG_MASK) >> E1000_MDIC_REG_SHIFT);
    //     return -E1000_ERR_PHY;
    // }
    if (mdic & E1000_MDIC_REG_MASK) >> E1000_MDIC_REG_SHIFT != offset {
        return Err(format!(
                "MDI write offset error - requested {}, returned {}",
                offset,
                (mdic & E1000_MDIC_REG_MASK) >> E1000_MDIC_REG_SHIFT,
        ));
    }

    // /* Allow some time after each MDIC transaction to avoid
    //  * reading duplicate data in the next MDIC transaction.
    //  */
    // if (hw->mac.type == e1000_pch2lan)
    //     usec_delay_irq(100);
    if adapter.hw.mac.mac_type == MacType::Mac_pch2lan {
        do_usec_delay(100);
    }
    // return E1000_SUCCESS;
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

    // Convenience closure
    // let mut release_if_locked = || if !locked {
    //     adapter
    //         .hw
    //         .phy
    //         .ops
    //         .release
    //         .ok_or("No function".to_string())
    //         .and_then(|f| {
    //             f(adapter);
    //             Ok(())
    //         })
    // } else {
    //     Ok(())
    // };
    fn release_if_locked(adapter: &mut Adapter, locked: bool) -> AdResult {
        if !locked {
            adapter.phy_release()
        } else {
            Ok(())
        }
    };

    // s32 ret_val;
    // u16 page = BM_PHY_REG_PAGE(offset);
    // u16 reg = BM_PHY_REG_NUM(offset);
    // u32 phy_addr = hw->phy.addr = e1000_get_phy_addr_for_hv_page(page);
    // DEBUGFUNC("__e1000_read_phy_reg_hv");
    let mut page: u16 = BM_PHY_REG_PAGE(offset);
    let reg: u16 = BM_PHY_REG_NUM(offset);
    let phy_addr: u32 = get_phy_addr_for_hv_page(page as u32);
    adapter.hw.phy.addr = phy_addr;

    // if (!locked) {
    //     ret_val = hw->phy.ops.acquire(hw);
    //     if (ret_val)
    //         return ret_val;
    // }
    if !locked {
        try!(adapter.phy_acquire());
    }

    // /* Page 800 works differently than the rest so it has its own func */
    // if (page == BM_WUC_PAGE) {
    //     ret_val = e1000_access_phy_wakeup_reg_bm(hw, offset, data,
    //     					 TRUE, page_set);
    //     goto out;
    // }
    if page == BM_WUC_PAGE as u16 {
        let res = access_phy_wakeup_reg_bm(adapter, offset, data, true, page_set);
        return release_if_locked(adapter, locked).and(res);
    }

    // if (page > 0 && page < HV_INTC_FC_PAGE_START) {
    //     ret_val = e1000_access_phy_debug_regs_hv(hw, offset,
    //     					 data, TRUE);
    //     goto out;
    // }
    if page > 0 && page < HV_INTC_FC_PAGE_START as u16 {
        let res = access_phy_debug_regs_hv(adapter, offset, data, true);
        return release_if_locked(adapter, locked).and(res);
    }
    // if (!page_set) {
    //     if (page == HV_INTC_FC_PAGE_START)
    //         page = 0;

    //     if (reg > MAX_PHY_MULTI_PAGE_REG) {
    //         /* Page is shifted left, PHY expects (page x 32) */
    //         ret_val = e1000_set_page_igp(hw,
    //     				 (page << IGP_PAGE_SHIFT));

    //         hw->phy.addr = phy_addr;

    //         if (ret_val)
    //     	goto out;
    //     }
    // }
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
    // DEBUGOUT3("reading PHY page %d (or 0x%x shifted) reg 0x%x\n", page,
    //           page << IGP_PAGE_SHIFT, reg);
    // ret_val = e1000_read_phy_reg_mdic(hw, MAX_PHY_REG_ADDRESS & reg,
    //     			      data);
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
    // e1000_verbose_println!("Reading PHY result: 0x{:x}", data);
    // out:
    // if (!locked)
    //     hw->phy.ops.release(hw);
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

    // s32 ret_val;
    // u16 page = BM_PHY_REG_PAGE(offset);
    // u16 reg = BM_PHY_REG_NUM(offset);
    // u32 phy_addr = hw->phy.addr = e1000_get_phy_addr_for_hv_page(page);
    let mut data = data;
    let mut page: u16 = BM_PHY_REG_PAGE(offset);
    let reg: u16 = BM_PHY_REG_NUM(offset);
    let phy_addr: u32 = get_phy_addr_for_hv_page(page as u32);
    adapter.hw.phy.addr = phy_addr;

    // DEBUGFUNC("__e1000_write_phy_reg_hv");

    // if (!locked) {
    //     ret_val = hw->phy.ops.acquire(hw);
    //     if (ret_val)
    //         return ret_val;
    // }
    if !locked {
        try!(adapter.phy_acquire());
    }
    // /* Page 800 works differently than the rest so it has its own func */
    // if (page == BM_WUC_PAGE) {
    //     ret_val = e1000_access_phy_wakeup_reg_bm(hw, offset, &data,
    //     					 FALSE, page_set);
    //     goto out;
    // }
    if page == BM_WUC_PAGE as u16 {
        let res = access_phy_wakeup_reg_bm(adapter, offset, &mut data, false, page_set);
        return release_if_locked(adapter, locked).and(res);
    }
    // if (page > 0 && page < HV_INTC_FC_PAGE_START) {
    //     ret_val = e1000_access_phy_debug_regs_hv(hw, offset,
    //     					 &data, FALSE);
    //     goto out;
    // }
    if page > 0 && page < HV_INTC_FC_PAGE_START as u16 {
        let res = access_phy_debug_regs_hv(adapter, offset, &mut data, false);
        return release_if_locked(adapter, locked).and(res);
    }

    // if (!page_set) {
    //     if (page == HV_INTC_FC_PAGE_START)
    //         page = 0;

    //     /* Workaround MDIO accesses being disabled after entering IEEE
    //      * Power Down (when bit 11 of the PHY Control register is set)
    //      */
    //     if ((hw->phy.type == e1000_phy_82578) &&
    //         (hw->phy.revision >= 1) &&
    //         (hw->phy.addr == 2) &&
    //         !(MAX_PHY_REG_ADDRESS & reg) &&
    //         (data & (1 << 11))) {
    //         u16 data2 = 0x7EFF;
    //         ret_val = e1000_access_phy_debug_regs_hv(hw,
    //     					     (1 << 6) | 0x3,
    //     					     &data2, FALSE);
    //         if (ret_val)
    //     	goto out;
    //     }

    //     if (reg > MAX_PHY_MULTI_PAGE_REG) {
    //         /* Page is shifted left, PHY expects (page x 32) */
    //         ret_val = e1000_set_page_igp(hw,
    //     				 (page << IGP_PAGE_SHIFT));

    //         hw->phy.addr = phy_addr;

    //         if (ret_val)
    //     	goto out;
    //     }
    // }
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
    // DEBUGOUT3("writing PHY page %d (or 0x%x shifted) reg 0x%x\n", page,
    //           page << IGP_PAGE_SHIFT, reg);
    e1000_verbose_println!(
        "Writing PHY page {} (or 0x{:x} shifted) reg 0x{:x}, data 0x{:x}",
        page,
        page << IGP_PAGE_SHIFT,
        reg, data
    );

    // ret_val = e1000_write_phy_reg_mdic(hw, MAX_PHY_REG_ADDRESS & reg,
    //     			       data);

    // out:
    // if (!locked)
    //     hw->phy.ops.release(hw);
    // return ret_val;

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
    // u32 kmrnctrlsta;
    let mut kmrnctrlsta: u32;

    // DEBUGFUNC("__e1000_read_kmrn_reg");

    // if (!locked) {
    //     s32 ret_val = E1000_SUCCESS;

    //     if (!hw->phy.ops.acquire)
    //         return E1000_SUCCESS;

    //     ret_val = hw->phy.ops.acquire(hw);
    //     if (ret_val)
    //         return ret_val;
    // }
    if !locked {
        if adapter.hw.phy.ops.acquire.is_none() {
            e1000_phy_println!("No function - return success");
            return Ok(())
        }
        try!(adapter.phy_acquire());
    }

    // kmrnctrlsta = ((offset << E1000_KMRNCTRLSTA_OFFSET_SHIFT) &
    //     	   E1000_KMRNCTRLSTA_OFFSET) | E1000_KMRNCTRLSTA_REN;
    // E1000_WRITE_REG(hw, E1000_KMRNCTRLSTA, kmrnctrlsta);
    // E1000_WRITE_FLUSH(hw);
    kmrnctrlsta = ((offset << E1000_KMRNCTRLSTA_OFFSET_SHIFT) & E1000_KMRNCTRLSTA_OFFSET) |
    E1000_KMRNCTRLSTA_REN;
    adapter.write_register(E1000_KMRNCTRLSTA, kmrnctrlsta);
    adapter.write_flush();

    // usec_delay(2);
    do_usec_delay(2);

    // kmrnctrlsta = E1000_READ_REG(hw, E1000_KMRNCTRLSTA);
    // *data = (u16)kmrnctrlsta;
    kmrnctrlsta = adapter.read_register(E1000_KMRNCTRLSTA);
    *data = kmrnctrlsta as u16;

    // if (!locked)
    //     hw->phy.ops.release(hw);
    if !locked {
        try!(adapter.phy_release());
    }
    // return E1000_SUCCESS;
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
    // u32 kmrnctrlsta;
    let mut kmrnctrlsta: u32 = 0;
    // DEBUGFUNC("e1000_write_kmrn_reg_generic");

    // if (!locked) {
    //     s32 ret_val = E1000_SUCCESS;

    //     if (!hw->phy.ops.acquire)
    //         return E1000_SUCCESS;

    //     ret_val = hw->phy.ops.acquire(hw);
    //     if (ret_val)
    //         return ret_val;
    // }
    if !locked {
        if adapter.hw.phy.ops.acquire.is_none() {
            e1000_phy_println!("No function - return success");
            return Ok(())
        }
        try!(adapter.phy_acquire());
    }
    // kmrnctrlsta = ((offset << E1000_KMRNCTRLSTA_OFFSET_SHIFT) &
    //     	   E1000_KMRNCTRLSTA_OFFSET) | data;
    // E1000_WRITE_REG(hw, E1000_KMRNCTRLSTA, kmrnctrlsta);
    // E1000_WRITE_FLUSH(hw);
    kmrnctrlsta = ((offset << E1000_KMRNCTRLSTA_OFFSET_SHIFT) & E1000_KMRNCTRLSTA_OFFSET) | data as u32;
    adapter.write_register(E1000_KMRNCTRLSTA, kmrnctrlsta);
    adapter.write_flush();

    // usec_delay(2);
    do_usec_delay(2);

    // if (!locked)
    //     hw->phy.ops.release(hw);
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
    // u32 phy_addr = 2;
    // if (page >= HV_INTC_FC_PAGE_START)
    // 	phy_addr = 1;
    // return phy_addr;
    let mut phy_addr: u32 = 2;
    if page >= HV_INTC_FC_PAGE_START {
        phy_addr = 1;
    }
    // e1000_phy_println!("page: {}, phy_addr: {}", page, phy_addr);
    phy_addr
}

/// e1000_get_phy_type_from_id - Get PHY type from id
/// @phy_id: phy_id read from the phy
///
/// Returns the phy type from the id.
// enum e1000_phy_type e1000_get_phy_type_from_id(u32 phy_id)
pub fn get_phy_type_from_id(phy_id: u32) -> PhyType {
    e1000_phy_println!();
    // enum e1000_phy_type phy_type = e1000_phy_unknown;

    // switch (phy_id) {
    let p = match phy_id {
        //     case M88E1000_I_PHY_ID:
        //     case M88E1000_E_PHY_ID:
        //     case M88E1111_I_PHY_ID:
        //     case M88E1011_I_PHY_ID:
        //     case M88E1543_E_PHY_ID:
        //     case M88E1512_E_PHY_ID:
        //     case I347AT4_E_PHY_ID:
        //     case M88E1112_E_PHY_ID:
        //     case M88E1340M_E_PHY_ID:
        //     phy_type = e1000_phy_m88;
        //     break;
        M88E1000_I_PHY_ID |
        M88E1000_E_PHY_ID |
        M88E1111_I_PHY_ID |
        M88E1011_I_PHY_ID |
        M88E1543_E_PHY_ID |
        M88E1512_E_PHY_ID |
        I347AT4_E_PHY_ID |
        M88E1112_E_PHY_ID |
        M88E1340M_E_PHY_ID => PhyType::Type_m88,
        //     case IGP01E1000_I_PHY_ID: /* IGP 1 & 2 share this */
        //     phy_type = e1000_phy_igp_2;
        //     break;
        IGP01E1000_I_PHY_ID => PhyType::Type_igp_2,
        //     case GG82563_E_PHY_ID:
        //     phy_type = e1000_phy_gg82563;
        //     break;
        GG82563_E_PHY_ID => PhyType::Type_gg82563,
        //     case IGP03E1000_E_PHY_ID:
        //     phy_type = e1000_phy_igp_3;
        //     break;
        IGP03E1000_E_PHY_ID => PhyType::Type_igp_3,
        //     case IFE_E_PHY_ID:
        //     case IFE_PLUS_E_PHY_ID:
        //     case IFE_C_E_PHY_ID:
        //     phy_type = e1000_phy_ife;
        //     break;
        IFE_E_PHY_ID |
        IFE_PLUS_E_PHY_ID |
        IFE_C_E_PHY_ID => PhyType::Type_ife,
        //     case BME1000_E_PHY_ID:
        //     case BME1000_E_PHY_ID_R2:
        //     phy_type = e1000_phy_bm;
        //     break;
        BME1000_E_PHY_ID |
        BME1000_E_PHY_ID_R2 => PhyType::Type_bm,
        //     case I82578_E_PHY_ID:
        //     phy_type = e1000_phy_82578;
        //     break;
        I82578_E_PHY_ID => PhyType::Type_82578,
        //     case I82577_E_PHY_ID:
        //     phy_type = e1000_phy_82577;
        //     break;
        I82577_E_PHY_ID => PhyType::Type_82577,
        //     case I82579_E_PHY_ID:
        //     phy_type = e1000_phy_82579;
        //     break;
        I82579_E_PHY_ID => PhyType::Type_82579,
        //     case I217_E_PHY_ID:
        //     phy_type = e1000_phy_i217;
        //     break;
        I217_E_PHY_ID => PhyType::Type_i217,
        //     case I82580_I_PHY_ID:
        //     phy_type = e1000_phy_82580;
        //     break;
        I82580_I_PHY_ID => PhyType::Type_82580,
        //     case I210_I_PHY_ID:
        //     phy_type = e1000_phy_i210;
        //     break;
        I210_I_PHY_ID => PhyType::Type_i210,
        //     default:
        //     phy_type = e1000_phy_unknown;
        //     break;
        _ => PhyType::Type_unknown,
        // }
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

    // if (phy->ops.check_reset_block) {
    //     ret_val = phy->ops.check_reset_block(hw);
    //     if (ret_val)
    //         return E1000_SUCCESS;
    // }
    match adapter.check_reset_block() {
        Ok(true) => return Ok(()),
        Ok(false) => (),
        Err(e) => eprintln!("{:?}", e)
    };

    // ret_val = phy->ops.acquire(hw);
    // if (ret_val)
    //     return ret_val;
    try!(adapter.phy_acquire());


    // ctrl = E1000_READ_REG(hw, E1000_CTRL);
    // E1000_WRITE_REG(hw, E1000_CTRL, ctrl | E1000_CTRL_PHY_RST);
    // E1000_WRITE_FLUSH(hw);

    adapter.set_register_bit(E1000_CTRL, E1000_CTRL_PHY_RST);
    adapter.write_flush();

    // usec_delay(phy->reset_delay_us);
    do_usec_delay(adapter.hw.phy.reset_delay_us as usize);

    // E1000_WRITE_REG(hw, E1000_CTRL, ctrl);
    // E1000_WRITE_FLUSH(hw);
    adapter.clear_register_bit(E1000_CTRL, E1000_CTRL_PHY_RST);
    adapter.write_flush();

    // usec_delay(150);
    do_usec_delay(150);

    // phy->ops.release(hw);
    try!(adapter.phy_release());

    // return phy->ops.get_cfg_done(hw);
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
    // s32 ret_val;
    // u16 phy_ctrl;

    // DEBUGFUNC("e1000_phy_sw_reset_generic");

    // if (!hw->phy.ops.read_reg)
    //     return E1000_SUCCESS;

    // ret_val = hw->phy.ops.read_reg(hw, PHY_CONTROL, &phy_ctrl);
    // if (ret_val)
    //     return ret_val;

    // phy_ctrl |= MII_CR_RESET;
    // ret_val = hw->phy.ops.write_reg(hw, PHY_CONTROL, phy_ctrl);
    // if (ret_val)
    //     return ret_val;

    // usec_delay(1);

    // return ret_val;
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

    // struct e1000_phy_info *phy = &hw->phy;
    // s32 ret_val;
    // u16 data;
    let mut data: u16 = 0;

    // DEBUGFUNC("e1000_check_polarity_m88");

    // ret_val = phy->ops.read_reg(hw, M88E1000_PHY_SPEC_STATUS, &data);

    // if (!ret_val)
    //     phy->cable_polarity = ((data & M88E1000_PSSR_REV_POLARITY)
    //     		       ? e1000_rev_polarity_reversed
    //     		       : e1000_rev_polarity_normal);

    try!(adapter.phy_read_reg(M88E1000_PHY_SPEC_STATUS, &mut data));
    adapter.hw.phy.cable_polarity = match btst!(data, M88E1000_PSSR_REV_POLARITY) {
        true => RevPolarity::Reversed,
        false => RevPolarity::Normal,
    };

    Ok(())
}

pub fn read_phy_reg_mdic(adapter: &mut Adapter, offset: u32, data: &mut u16) -> AdResult {
    /// Fatal on error

    // e1000_phy_println!();

    // 	struct e1000_phy_info *phy = &hw->phy;
    // 	u32 i, mdic = 0;
    let mut mdic: u32;

    // 	DEBUGFUNC("e1000_read_phy_reg_mdic");

    // 	if (offset > MAX_PHY_REG_ADDRESS) {
    // 		DEBUGOUT1("PHY Address %d is out of range\n", offset);
    // 		return -E1000_ERR_PARAM;
    // 	}

    if offset > MAX_PHY_REG_ADDRESS {
        return Err(format!("PHY address {} is out of range", offset));
    }

    /* Set up Op-code, Phy Address, and register offset in the MDI
     * Control register.  The MAC will take care of interfacing with the
     * PHY to retrieve the desired data.
     */
    // 	mdic = ((offset << E1000_MDIC_REG_SHIFT) |
    // 		(phy->addr << E1000_MDIC_PHY_SHIFT) |
    // 		(E1000_MDIC_OP_READ));

    mdic = (offset << E1000_MDIC_REG_SHIFT) | (adapter.hw.phy.addr << E1000_MDIC_PHY_SHIFT) |
        E1000_MDIC_OP_READ;

    // 	E1000_WRITE_REG(hw, E1000_MDIC, mdic);

    do_write_register(adapter, E1000_MDIC, mdic);

    /* Poll the ready bit to see if the MDI read completed
     * Increasing the time out as testing showed failures with
     * the lower time out
     */
    // 	for (i = 0; i < (E1000_GEN_POLL_TIMEOUT * 3); i++) {
    // 		usec_delay_irq(50);
    // 		mdic = E1000_READ_REG(hw, E1000_MDIC);
    // 		if (mdic & E1000_MDIC_READY)
    // 			break;
    // 	}

    for i in 0..(E1000_GEN_POLL_TIMEOUT * 3) {
        do_usec_delay(50);
        mdic = do_read_register(adapter, E1000_MDIC);
        if btst!(mdic, E1000_MDIC_READY) {
            break;
        }
    }

    // 	if (!(mdic & E1000_MDIC_READY)) {
    // 		DEBUGOUT("MDI Read did not complete\n");
    // 		return -E1000_ERR_PHY;
    // 	}

    if !btst!(mdic, E1000_MDIC_READY) {
        return Err("MDI read did not complete".into());
    }

    // 	if (mdic & E1000_MDIC_ERROR) {
    // 		DEBUGOUT("MDI Error\n");
    // 		return -E1000_ERR_PHY;
    // 	}

    if btst!(mdic, E1000_MDIC_ERROR) {
        return Err("MDI error".into());
    }

    // 	if (((mdic & E1000_MDIC_REG_MASK) >> E1000_MDIC_REG_SHIFT) != offset) {
    // 		DEBUGOUT2("MDI Read offset error - requested %d, returned %d\n",
    // 			  offset,
    // 			  (mdic & E1000_MDIC_REG_MASK) >> E1000_MDIC_REG_SHIFT);
    // 		return -E1000_ERR_PHY;
    // 	}
    // 	*data = (u16) mdic;

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
    // 	if (hw->mac.type == e1000_pch2lan)
    // 		usec_delay_irq(100);

    if adapter.hw.mac.mac_type == MacType::Mac_pch2lan {
        do_usec_delay(100);
    }

    // 	return E1000_SUCCESS;
    // }

    Ok(())
}

pub fn has_link_generic(
    adapter: &mut Adapter,
    iterations: u32,
    usec_interval: u32,
    success: &mut bool,
) -> AdResult {
    e1000_phy_println!();
    // s32 ret_val = E1000_SUCCESS;
    // u16 i, phy_status;
    // DEBUGFUNC("e1000_phy_has_link_generic");

    // if (!hw->phy.ops.read_reg)
    //     return E1000_SUCCESS;

    let mut i = 0;
    let mut phy_status: u16 = 0;

    let mut res = Ok(());
    if let Some(read_reg) = adapter.hw.phy.ops.read_reg {

        // for (i = 0; i < iterations; i++) {
        //     /* Some PHYs require the PHY_STATUS register to be read
        //      * twice due to the link bit being sticky.  No harm doing
        //      * it across the board.
        //      */
        //     ret_val = hw->phy.ops.read_reg(hw, PHY_STATUS, &phy_status);
        //     if (ret_val) {
        //         /* If the first read fails, another entity may have
        //          * ownership of the resources, wait and try again to
        //          * see if they have relinquished the resources yet.
        //          */
        //         if (usec_interval >= 1000)
        //     	msec_delay(usec_interval/1000);
        //         else
        //     	usec_delay(usec_interval);
        //     }
        //     ret_val = hw->phy.ops.read_reg(hw, PHY_STATUS, &phy_status);
        //     if (ret_val)
        //         break;
        //     if (phy_status & MII_SR_LINK_STATUS)
        //         break;
        //     if (usec_interval >= 1000)
        //         msec_delay(usec_interval/1000);
        //     else
        //         usec_delay(usec_interval);
        // }
        // *success = (i < iterations);
        // return ret_val;
        while i < iterations {
            if read_reg(adapter, PHY_STATUS, &mut phy_status).is_err() {
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

    e1000_phy_println!("Get phy status success: {}/{}", i, iterations);

    res
}

pub fn check_downshift_generic(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();
    // struct e1000_phy_info *phy = &hw->phy;
    // s32 ret_val;
    // u16 phy_data, offset, mask;
    let mut phy_data: u16 = 0;
    let mut offset: u16 = 0;
    let mut mask: u16 = 0;

    // DEBUGFUNC("e1000_check_downshift_generic");

    // switch (phy->type) {
    //     case e1000_phy_i210:
    //     case e1000_phy_m88:
    //     case e1000_phy_gg82563:
    //     case e1000_phy_bm:
    //     case e1000_phy_82578:
    //     offset = M88E1000_PHY_SPEC_STATUS;
    //     mask = M88E1000_PSSR_DOWNSHIFT;
    //     break;
    //     case e1000_phy_igp:
    //     case e1000_phy_igp_2:
    //     case e1000_phy_igp_3:
    //     offset = IGP01E1000_PHY_LINK_HEALTH;
    //     mask = IGP01E1000_PLHR_SS_DOWNGRADE;
    //     break;
    //     default:
    //     /* speed downshift not supported */
    //     phy->speed_downgraded = FALSE;
    //     return E1000_SUCCESS;
    // }
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
    // ret_val = phy->ops.read_reg(hw, offset, &phy_data);
    try!(adapter.phy_read_reg(offset as u32, &mut phy_data));

    // if (!ret_val)
    //     phy->speed_downgraded = !!(phy_data & mask);

    adapter.hw.phy.speed_downgraded = btst!(phy_data, mask);

    // return ret_val;
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
    // s32 e1000_set_page_igp(struct e1000_hw *hw, u16 page)
    // {
    // 	DEBUGFUNC("e1000_set_page_igp");
    // 	DEBUGOUT1("Setting page 0x%x\n", page);
    // 	hw->phy.addr = 1;
    // 	return e1000_write_phy_reg_mdic(hw, IGP01E1000_PHY_PAGE_SELECT, page);
    // }
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
    // s32 ret_val;
    // u32 addr_reg;
    // u32 data_reg;

    // DEBUGFUNC("e1000_access_phy_debug_regs_hv");

    // /* This takes care of the difference with desktop vs mobile phy */
    // addr_reg = ((hw->phy.type == e1000_phy_82578) ?
    //     	I82578_ADDR_REG : I82577_ADDR_REG);
    // data_reg = addr_reg + 1;

    // /* All operations in this function are phy address 2 */
    // hw->phy.addr = 2;

    // /* masking with 0x3F to remove the page from offset */
    // ret_val = e1000_write_phy_reg_mdic(hw, addr_reg, (u16)offset & 0x3F);
    // if (ret_val) {
    //     DEBUGOUT("Could not write the Address Offset port register\n");
    //     return ret_val;
    // }

    // /* Read or write the data value next */
    // if (read)
    //     ret_val = e1000_read_phy_reg_mdic(hw, data_reg, data);
    // else
    //     ret_val = e1000_write_phy_reg_mdic(hw, data_reg, *data);

    // if (ret_val)
    //     DEBUGOUT("Could not access the Data port register\n");

    // return ret_val;
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
    // DEBUGFUNC("e1000_access_phy_wakeup_reg_bm");
    e1000_verbose_println!();

    // s32 ret_val;
    // u16 reg = BM_PHY_REG_NUM(offset);
    // u16 page = BM_PHY_REG_PAGE(offset);
    // u16 phy_reg = 0;
    let mut reg: u16 = BM_PHY_REG_NUM(offset);
    let mut page: u16 = BM_PHY_REG_PAGE(offset);
    let mut phy_reg: u16 = 0;


    /* Gig must be disabled for MDIO accesses to Host Wakeup reg page */
    // if ((hw->mac.type == e1000_pchlan) &&
    //     (!(E1000_READ_REG(hw, E1000_PHY_CTRL) & E1000_PHY_CTRL_GBE_DISABLE)))
    //     DEBUGOUT1("Attempting to access page %d while gig enabled.\n",
    //     	  page);
    if adapter.hw.mac.mac_type == MacType::Mac_pchlan {
        incomplete_return!();
    }
    // if (!page_set) {
    //     /* Enable access to PHY wakeup registers */
    //     ret_val = e1000_enable_phy_wakeup_reg_access_bm(hw, &phy_reg);
    //     if (ret_val) {
    //         DEBUGOUT("Could not enable PHY wakeup reg access\n");
    //         return ret_val;
    //     }
    // }
    if !page_set {
        try!(enable_phy_wakeup_reg_access_bm(adapter, &mut phy_reg));
    }

    // DEBUGOUT2("Accessing PHY page %d reg 0x%x\n", page, reg);

    /* Write the Wakeup register page offset value using opcode 0x11 */
    // ret_val = e1000_write_phy_reg_mdic(hw, BM_WUC_ADDRESS_OPCODE, reg);
    // if (ret_val) {
    //     DEBUGOUT1("Could not write address opcode to page %d\n", page);
    //     return ret_val;
    // }
    try!(write_phy_reg_mdic(adapter, BM_WUC_ADDRESS_OPCODE, reg));

    // if (read) {
    //     /* Read the Wakeup register page value using opcode 0x12 */
    //     ret_val = e1000_read_phy_reg_mdic(hw, BM_WUC_DATA_OPCODE,
    //     				  data);
    // } else {
    //     /* Write the Wakeup register page value using opcode 0x12 */
    //     ret_val = e1000_write_phy_reg_mdic(hw, BM_WUC_DATA_OPCODE,
    //     				   *data);
    // }
    // if (ret_val) {
    //     DEBUGOUT2("Could not access PHY reg %d.%d\n", page, reg);
    //     return ret_val;
    // }
    if read {
        try!(read_phy_reg_mdic(adapter, BM_WUC_DATA_OPCODE, data));
    } else {
        try!(write_phy_reg_mdic(adapter, BM_WUC_DATA_OPCODE, *data));
    }

    // if (!page_set)
    //     ret_val = e1000_disable_phy_wakeup_reg_access_bm(hw, &phy_reg);
    if !page_set {
        try!(disable_phy_wakeup_reg_access_bm(adapter, &mut phy_reg));
    }

    // return ret_val;
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
    // struct e1000_phy_info *phy = &hw->phy;
    // s32 ret_val;
    // u16 data;
    // DEBUGFUNC("e1000_check_polarity_82577");
    // ret_val = phy->ops.read_reg(hw, I82577_PHY_STATUS_2, &data);
    // if (!ret_val)
    //     phy->cable_polarity = ((data & I82577_PHY_STATUS2_REV_POLARITY)
    //     		       ? e1000_rev_polarity_reversed
    //     		       : e1000_rev_polarity_normal);
    // return ret_val;
    incomplete_return!();
}

/// e1000_phy_force_speed_duplex_82577 - Force speed/duplex for I82577 PHY
/// @hw: pointer to the HW structure
///
/// Calls the PHY setup function to force speed and duplex.
pub fn phy_force_speed_duplex_82577(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();
    // struct e1000_phy_info *phy = &hw->phy;
    // s32 ret_val;
    // u16 phy_data;
    // bool link;

    // DEBUGFUNC("e1000_phy_force_speed_duplex_82577");

    // ret_val = phy->ops.read_reg(hw, PHY_CONTROL, &phy_data);
    // if (ret_val)
    //     return ret_val;

    // e1000_phy_force_speed_duplex_setup(hw, &phy_data);

    // ret_val = phy->ops.write_reg(hw, PHY_CONTROL, phy_data);
    // if (ret_val)
    //     return ret_val;

    // usec_delay(1);

    // if (phy->autoneg_wait_to_complete) {
    //     DEBUGOUT("Waiting for forced speed/duplex link on 82577 phy\n");

    //     ret_val = e1000_phy_has_link_generic(hw, PHY_FORCE_LIMIT,
    //     				     100000, &link);
    //     if (ret_val)
    //         return ret_val;

    //     if (!link)
    //         DEBUGOUT("Link taking longer than expected.\n");

    //     /* Try once more */
    //     ret_val = e1000_phy_has_link_generic(hw, PHY_FORCE_LIMIT,
    //     				     100000, &link);
    // }

    // return ret_val;
    incomplete_return!();
}

/// e1000_get_cable_length_82577 - Determine cable length for 82577 PHY
/// @hw: pointer to the HW structure
///
/// Reads the diagnostic status register and verifies result is valid before
/// placing it in the phy_cable_length field.
pub fn get_cable_length_82577(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();
    // struct e1000_phy_info *phy = &hw->phy;
    // s32 ret_val;
    // u16 phy_data, length;

    // DEBUGFUNC("e1000_get_cable_length_82577");

    // ret_val = phy->ops.read_reg(hw, I82577_PHY_DIAG_STATUS, &phy_data);
    // if (ret_val)
    //     return ret_val;

    // length = ((phy_data & I82577_DSTATUS_CABLE_LENGTH) >>
    //           I82577_DSTATUS_CABLE_LENGTH_SHIFT);

    // if (length == E1000_CABLE_LENGTH_UNDEFINED)
    //     return -E1000_ERR_PHY;

    // phy->cable_length = length;

    // return E1000_SUCCESS;
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
    // struct e1000_phy_info *phy = &hw->phy;
    // s32 ret_val;
    // u16 data;
    // bool link;
    let mut data: u16 = 0;
    let mut link: bool = false;

    // DEBUGFUNC("e1000_get_phy_info_82577");

    // ret_val = e1000_phy_has_link_generic(hw, 1, 0, &link);
    // if (ret_val)
    //     return ret_val;
    try!(has_link_generic(adapter, 1, 0, &mut link));

    // if (!link) {
    //     DEBUGOUT("Phy info is only valid if link is up\n");
    //     return -E1000_ERR_CONFIG;
    // }
    if !link {
        return Err("Phy info is only valid if link is up".to_string());
    }

    // phy->polarity_correction = TRUE;
    adapter.hw.phy.polarity_correction = true;

    // ret_val = e1000_check_polarity_82577(hw);
    // if (ret_val)
    //     return ret_val;
    try!(check_polarity_82577(adapter));

    // ret_val = phy->ops.read_reg(hw, I82577_PHY_STATUS_2, &data);
    // if (ret_val)
    //     return ret_val;
    try!(adapter.phy_read_reg(I82577_PHY_STATUS_2, &mut data));

    // phy->is_mdix = !!(data & I82577_PHY_STATUS2_MDIX);
    adapter.hw.phy.is_mdix = btst!(data, I82577_PHY_STATUS2_MDIX);

    // if ((data & I82577_PHY_STATUS2_SPEED_MASK) ==
    //     I82577_PHY_STATUS2_SPEED_1000MBPS) {
    //     ret_val = hw->phy.ops.get_cable_length(hw);
    //     if (ret_val)
    //         return ret_val;

    //     ret_val = phy->ops.read_reg(hw, PHY_1000T_STATUS, &data);
    //     if (ret_val)
    //         return ret_val;

    //     phy->local_rx = (data & SR_1000T_LOCAL_RX_STATUS)
    //         ? e1000_1000t_rx_status_ok
    //         : e1000_1000t_rx_status_not_ok;

    //     phy->remote_rx = (data & SR_1000T_REMOTE_RX_STATUS)
    //         ? e1000_1000t_rx_status_ok
    //         : e1000_1000t_rx_status_not_ok;
    // } else {
    //     phy->cable_length = E1000_CABLE_LENGTH_UNDEFINED;
    //     phy->local_rx = e1000_1000t_rx_status_undefined;
    //     phy->remote_rx = e1000_1000t_rx_status_undefined;
    // }
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
    // return E1000_SUCCESS;
    Ok(())
}

/// e1000_copper_link_setup_82577 - Setup 82577 PHY for copper link
/// @hw: pointer to the HW structure
///
/// Sets up Carrier-sense on Transmit and downshift values.
pub fn copper_link_setup_82577(adapter: &mut Adapter) -> AdResult {
    e1000_phy_println!();
    // s32 ret_val;
    // u16 phy_data;
    let mut phy_data: u16 = 0;

    // DEBUGFUNC("e1000_copper_link_setup_82577");

    // if (hw->phy.type == e1000_phy_82580) {
    //     ret_val = hw->phy.ops.reset(hw);
    //     if (ret_val) {
    //         DEBUGOUT("Error resetting the PHY.\n");
    //         return ret_val;
    //     }
    // }
    if adapter.hw.phy.phy_type == PhyType::Type_82580 {
        try!(adapter.phy_reset());
    }

    /* Enable CRS on Tx. This must be set for half-duplex operation. */
    // ret_val = hw->phy.ops.read_reg(hw, I82577_CFG_REG, &phy_data);
    // if (ret_val)
    //     return ret_val;
    try!(adapter.phy_read_reg(I82577_CFG_REG, &mut phy_data));

    // phy_data |= I82577_CFG_ASSERT_CRS_ON_TX;
    phy_data |= I82577_CFG_ASSERT_CRS_ON_TX;

    /* Enable downshift */
    // phy_data |= I82577_CFG_ENABLE_DOWNSHIFT;
    phy_data |= I82577_CFG_ENABLE_DOWNSHIFT;

    // ret_val = hw->phy.ops.write_reg(hw, I82577_CFG_REG, phy_data);
    // if (ret_val)
    //     return ret_val;
    try!(adapter.phy_write_reg(I82577_CFG_REG, phy_data));

    /* Set MDI/MDIX mode */
    // ret_val = hw->phy.ops.read_reg(hw, I82577_PHY_CTRL_2, &phy_data);
    // if (ret_val)
    //     return ret_val;
    try!(adapter.phy_read_reg(I82577_PHY_CTRL_2, &mut phy_data));

    // phy_data &= ~I82577_PHY_CTRL2_MDIX_CFG_MASK;
    phy_data &= !(I82577_PHY_CTRL2_MDIX_CFG_MASK);

    /* Options:
     *   0 - Auto (default)
     *   1 - MDI mode
     *   2 - MDI-X mode
     */
    // switch (hw->phy.mdix) {
    //     case 1:
    //     break;
    //     case 2:
    //     phy_data |= I82577_PHY_CTRL2_MANUAL_MDIX;
    //     break;
    //     case 0:
    //     default:
    //     phy_data |= I82577_PHY_CTRL2_AUTO_MDI_MDIX;
    //     break;
    // }
    match adapter.hw.phy.mdix {
        1 => (),
        2 => phy_data |= I82577_PHY_CTRL2_MANUAL_MDIX,
        _ => phy_data |= I82577_PHY_CTRL2_AUTO_MDI_MDIX,
    }

    // ret_val = hw->phy.ops.write_reg(hw, I82577_PHY_CTRL_2, phy_data);
    // if (ret_val)
    //     return ret_val;
    try!(adapter.phy_write_reg(I82577_PHY_CTRL_2, phy_data));

    // return e1000_set_master_slave_mode(hw);
    set_master_slave_mode(adapter)
}

/// e1000_set_master_slave_mode - Setup PHY for Master/slave mode
/// @hw: pointer to the HW structure
///
/// Sets up Master/slave mode
pub fn set_master_slave_mode(adapter: &mut Adapter) -> AdResult {
    // s32 ret_val;
    // u16 phy_data;
    let mut phy_data: u16 = 0;

    /* Resolve Master/Slave mode */
    // ret_val = hw->phy.ops.read_reg(hw, PHY_1000T_CTRL, &phy_data);
    // if (ret_val)
    //     return ret_val;
    try!(adapter.phy_read_reg(PHY_1000T_CTRL, &mut phy_data));

    /* load defaults for future use */
    // hw->phy.original_ms_type = (phy_data & CR_1000T_MS_ENABLE) ?
    //     ((phy_data & CR_1000T_MS_VALUE) ?
    //      e1000_ms_force_master :
    //      e1000_ms_force_slave) : e1000_ms_auto;
    adapter.hw.phy.original_ms_type = match btst!(phy_data, CR_1000T_MS_ENABLE) {
        true => match btst!(phy_data, CR_1000T_MS_VALUE) {
            true => MsType::ForceMaster,
            false => MsType::ForceSlave
        }
        false => MsType::Auto
    };

    // switch (hw->phy.ms_type) {
    //     case e1000_ms_force_master:
    //     phy_data |= (CR_1000T_MS_ENABLE | CR_1000T_MS_VALUE);
    //     break;
    //     case e1000_ms_force_slave:
    //     phy_data |= CR_1000T_MS_ENABLE;
    //     phy_data &= ~(CR_1000T_MS_VALUE);
    //     break;
    //     case e1000_ms_auto:
    //     phy_data &= ~CR_1000T_MS_ENABLE;
    //     /* fall-through */
    //     default:
    //     break;
    // }
    match adapter.hw.phy.ms_type {
        MsType::ForceMaster => phy_data |= CR_1000T_MS_ENABLE | CR_1000T_MS_VALUE,
        MsType::ForceSlave => {
            phy_data |= CR_1000T_MS_ENABLE;
            phy_data &= !(CR_1000T_MS_VALUE);
        },
        MsType::Auto => phy_data &= !(CR_1000T_MS_ENABLE),
        _ => ()
    }
    // return hw->phy.ops.write_reg(hw, PHY_1000T_CTRL, phy_data);
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
    // s32 ret_val;
    // u16 temp;
    let mut temp: u16;

    // DEBUGFUNC("e1000_enable_phy_wakeup_reg_access_bm");

    // if (!phy_reg)
    // 	return -E1000_ERR_PARAM;

    /* All page select, port ctrl and wakeup registers use phy address 1 */
    // hw->phy.addr = 1;
    adapter.hw.phy.addr = 1;

    /* Select Port Control Registers page */
    // ret_val = e1000_set_page_igp(hw, (BM_PORT_CTRL_PAGE << IGP_PAGE_SHIFT));
    // if (ret_val) {
    // 	DEBUGOUT("Could not set Port Control page\n");
    // 	return ret_val;
    // }
    try!(set_page_igp(adapter, (BM_PORT_CTRL_PAGE << IGP_PAGE_SHIFT) as u16));

    // ret_val = e1000_read_phy_reg_mdic(hw, BM_WUC_ENABLE_REG, phy_reg);
    // if (ret_val) {
    // 	DEBUGOUT2("Could not read PHY register %d.%d\n",
    // 		  BM_PORT_CTRL_PAGE, BM_WUC_ENABLE_REG);
    // 	return ret_val;
    // }
    try!(read_phy_reg_mdic(adapter, BM_WUC_ENABLE_REG, phy_reg));

    /* Enable both PHY wakeup mode and Wakeup register page writes.
     * Prevent a power state change by disabling ME and Host PHY wakeup.
     */
    // temp = *phy_reg;
    // temp |= BM_WUC_ENABLE_BIT;
    // temp &= ~(BM_WUC_ME_WU_BIT | BM_WUC_HOST_WU_BIT);
    temp = *phy_reg;
    temp |= BM_WUC_ENABLE_BIT as u16;
    temp &= !((BM_WUC_ME_WU_BIT | BM_WUC_HOST_WU_BIT) as u16);

    // ret_val = e1000_write_phy_reg_mdic(hw, BM_WUC_ENABLE_REG, temp);
    // if (ret_val) {
    // 	DEBUGOUT2("Could not write PHY register %d.%d\n",
    // 		  BM_PORT_CTRL_PAGE, BM_WUC_ENABLE_REG);
    // 	return ret_val;
    // }
    try!(write_phy_reg_mdic(adapter, BM_WUC_ENABLE_REG, temp));

    /* Select Host Wakeup Registers page - caller now able to write
     * registers on the Wakeup registers page
     */
    // return e1000_set_page_igp(hw, (BM_WUC_PAGE << IGP_PAGE_SHIFT));
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
    // s32 ret_val;

    // DEBUGFUNC("e1000_disable_phy_wakeup_reg_access_bm");

    // if (!phy_reg)
    //     return -E1000_ERR_PARAM;

    /* Select Port Control Registers page */
    // ret_val = e1000_set_page_igp(hw, (BM_PORT_CTRL_PAGE << IGP_PAGE_SHIFT));
    // if (ret_val) {
    //     DEBUGOUT("Could not set Port Control page\n");
    //     return ret_val;
    // }
    try!(set_page_igp(adapter, (BM_PORT_CTRL_PAGE << IGP_PAGE_SHIFT) as u16));

    /* Restore 769.17 to its original value */
    // ret_val = e1000_write_phy_reg_mdic(hw, BM_WUC_ENABLE_REG, *phy_reg);
    // if (ret_val)
    //     DEBUGOUT2("Could not restore PHY register %d.%d\n",
    //     	  BM_PORT_CTRL_PAGE, BM_WUC_ENABLE_REG);
    // return ret_val;
    write_phy_reg_mdic(adapter, BM_WUC_ENABLE_REG, *phy_reg)
}
