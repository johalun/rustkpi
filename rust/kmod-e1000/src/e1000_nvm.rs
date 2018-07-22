
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
use e1000_osdep::*;
use e1000_regs::*;


pub fn read_mac_addr_generic(adapter: &mut Adapter) -> AdResult {
    e1000_println!();
    // u32 rar_high;
    // u32 rar_low;
    // u16 i;

    // rar_high = E1000_READ_REG(hw, E1000_RAH(0));
    // rar_low = E1000_READ_REG(hw, E1000_RAL(0));

    // for (i = 0; i < E1000_RAL_MAC_ADDR_LEN; i++)
    // 	hw->mac.perm_addr[i] = (u8)(rar_low >> (i*8));

    // for (i = 0; i < E1000_RAH_MAC_ADDR_LEN; i++)
    // 	hw->mac.perm_addr[i+4] = (u8)(rar_high >> (i*8));

    // for (i = 0; i < ETH_ADDR_LEN; i++)
    // 	hw->mac.addr[i] = hw->mac.perm_addr[i];

    // return E1000_SUCCESS;

    let rar_high: u32 = do_read_register(adapter, E1000_RAH(0));
    let rar_low: u32 = do_read_register(adapter, E1000_RAL(0));

    for i in 0..E1000_RAL_MAC_ADDR_LEN as usize {
        adapter.hw.mac.perm_addr[i] = (rar_low >> (i * 8)) as u8;
    }

    for i in 0..E1000_RAH_MAC_ADDR_LEN as usize {
        adapter.hw.mac.perm_addr[i + 4] = (rar_high >> (i * 8)) as u8;
    }

    for i in 0..kernel::sys::iflib_sys::ETH_ADDR_LEN as usize {
        adapter.hw.mac.addr[i] = adapter.hw.mac.perm_addr[i];
    }

    Ok(())
}

pub fn reload_nvm_generic(adapter: &mut Adapter) {
    // e1000_println!();
    incomplete!();
}

pub fn acquire_nvm_generic(adapter: &mut Adapter) -> AdResult {
    // e1000_println!();

    // u32 eecd = E1000_READ_REG(hw, E1000_EECD);
    // s32 timeout = E1000_NVM_GRANT_ATTEMPTS;

    let mut eecd: u32 = do_read_register(adapter, E1000_EECD);
    let mut timeout: i32 = E1000_NVM_GRANT_ATTEMPTS as i32;

    // E1000_WRITE_REG(hw, E1000_EECD, eecd | E1000_EECD_REQ);
    // eecd = E1000_READ_REG(hw, E1000_EECD);

    // e1000_println!("eecd before 0x{:b}", eecd);
    // e1000_println!("write 0x{:b}", eecd | E1000_EECD_REQ);
    do_write_register(adapter, E1000_EECD, eecd | E1000_EECD_REQ);
    eecd = do_read_register(adapter, E1000_EECD);
    // e1000_println!("eecd after  0x{:b}", eecd);

    // while (timeout) {
    // 	if (eecd & E1000_EECD_GNT)
    // 		break;
    // 	usec_delay(5);
    // 	eecd = E1000_READ_REG(hw, E1000_EECD);
    // 	timeout--;
    // }

    while timeout > 0 {
        if eecd & E1000_EECD_GNT > 0 {
            break;
        }
        do_usec_delay(5);
        eecd = do_read_register(adapter, E1000_EECD);
        timeout -= 1;
    }

    // if (!timeout) {
    // 	eecd &= ~E1000_EECD_REQ;
    // 	E1000_WRITE_REG(hw, E1000_EECD, eecd);
    // 	DEBUGOUT("Could not acquire NVM grant\n");
    // 	return -E1000_ERR_NVM;
    // }

    if timeout == 0 {
        eecd &= !E1000_EECD_REQ;
        do_write_register(adapter, E1000_EECD, eecd);
        Err("Could not acquire NVM grant".to_string())
    } else {
        // e1000_println!("Got NVM grant with {} timeouts to spare", timeout);
        Ok(())
    }
}

pub fn read_nvm_microwire(
    adapter: &mut Adapter,
    offset: u16,
    words: u16,
    data: &mut [u16],
) -> AdResult {
    // e1000_println!("offset {}, words {}", offset, words);

    // let mut ret = 0;
    let read_opcode: u8 = NVM_READ_OPCODE_MICROWIRE as u8;

    /* A check for invalid values:  offset too large, too many words,
     * and not enough words.
     */
    // 	if ((offset >= nvm->word_size) || (words > (nvm->word_size - offset)) ||
    // 	    (words == 0)) {
    // 		DEBUGOUT("nvm parameter(s) out of bounds\n");
    // 		return -E1000_ERR_NVM;
    // 	}
    let word_size = adapter.hw.nvm.word_size;
    if offset >= word_size || words > (word_size - offset) || words == 0 {
        return Err("nvm parameter out of bounds".into());
    }

    // 	ret_val = nvm->ops.acquire(hw);
    // 	if (ret_val)
    // 		return ret_val;

    // let acquire = try!(adapter.hw.nvm.ops.acquire.ok_or("No function".into()));

    let acquire = try!(adapter.hw.nvm.ops.acquire.ok_or("No function".to_string()));
    let release = try!(adapter.hw.nvm.ops.release.ok_or("No function".to_string()));


    try!(acquire(adapter));

    // 	ret_val = e1000_ready_nvm_eeprom(hw);
    // 	if (ret_val)
    // 		goto release;

    let _ = ready_nvm_eeprom(adapter).or_else(|e| {
        release(adapter);
        return Err(e);
    });

    // 	for (i = 0; i < words; i++) {
    // 		/* Send the READ command (opcode + addr) */
    // 		e1000_shift_out_eec_bits(hw, read_opcode, nvm->opcode_bits);
    // 		e1000_shift_out_eec_bits(hw, (u16)(offset + i),
    // 					nvm->address_bits);

    // 		/* Read the data.  For microwire, each word requires the
    // 		 * overhead of setup and tear-down.
    // 		 */
    // 		data[i] = e1000_shift_in_eec_bits(hw, 16);
    // 		e1000_standby_nvm(hw);
    // 	}
    let opcode_bits = adapter.hw.nvm.opcode_bits;
    let address_bits = adapter.hw.nvm.address_bits;

    // e1000_println!("opcode:       {:032b}", read_opcode);
    // e1000_println!("opcode_bits:  {}", opcode_bits);
    // e1000_println!("address_bits: {}", address_bits);

    for i in 0..words as usize {
        // e1000_println!("send read command: {}", i);
        /* Send the READ command (opcode + addr) */
        shift_out_eec_bits(adapter, read_opcode as u16, opcode_bits);
        shift_out_eec_bits(adapter, offset + i as u16, address_bits);
        /* Read the data.  For microwire, each word requires the
         * overhead of setup and tear-down.
         */
        let value: u16 = shift_in_eec_bits(adapter, 16);
        // println!("Store {} to index {}", value, i);
        data[i] = value;
        standby_nvm(adapter);
    }
    release(adapter);
    Ok(())
}

pub fn release_nvm_generic(adapter: &mut Adapter) {
    // e1000_println!();

    let mut eecd: u32;

    // e1000_stop_nvm(hw);
    // eecd = E1000_READ_REG(hw, E1000_EECD);
    // eecd &= ~E1000_EECD_REQ;
    // E1000_WRITE_REG(hw, E1000_EECD, eecd);

    stop_nvm(adapter);

    eecd = do_read_register(adapter, E1000_EECD);
    eecd &= !E1000_EECD_REQ;
    do_write_register(adapter, E1000_EECD, eecd);

}

pub fn update_nvm_checksum_generic(adapter: &mut Adapter) -> AdResult {
    // e1000_println!();
    incomplete!();
    Ok(())
}

pub fn validate_nvm_checksum_generic(adapter: &mut Adapter) -> AdResult {
    e1000_println!();

    // for (i = 0; i < (NVM_CHECKSUM_REG + 1); i++) {
    // 	ret_val = hw->nvm.ops.read(hw, i, 1, &nvm_data);
    // 	if (ret_val) {
    // 		DEBUGOUT("NVM Read Error\n");
    // 		return ret_val;
    // 	}
    // 	checksum += nvm_data;
    // }

    let mut nvm_data: [u16; 1] = [0u16; 1];
    let mut checksum: u16 = 0;

    let read = try!(adapter.hw.nvm.ops.read.ok_or("No function".to_string()));

    for i in 0..(NVM_CHECKSUM_REG + 1) as u16 {
        try!(read(adapter, i, 1, &mut nvm_data));
        // println!("nvm_data {:?}", nvm_data);
        checksum += nvm_data[0];
    }

    // if (checksum != (u16) NVM_SUM) {
    // 	DEBUGOUT("NVM Checksum Invalid\n");
    // 	return -E1000_ERR_NVM;
    // }

    if checksum != NVM_SUM as u16 {
        return Err(format!(
            "NVM checksum error. Got 0x{:x}. Should be 0x{:x}.",
            checksum,
            NVM_SUM
        ));
    }
    Ok(())
}

pub fn write_nvm_microcode(adapter: &mut Adapter, arg2: u16, arg3: u16, arg4: &[u16]) -> AdResult {
    e1000_println!();
    incomplete_return!();
}

pub fn ready_nvm_eeprom(adapter: &mut Adapter) -> AdResult {
    // e1000_println!();

    // struct e1000_nvm_info *nvm = &hw->nvm;
    // u32 eecd = E1000_READ_REG(hw, E1000_EECD);
    // u8 spi_stat_reg;

    let mut eecd: u32 = do_read_register(adapter, E1000_EECD);

    // if (nvm->type == e1000_nvm_eeprom_microwire) {
    // 	/* Clear SK and DI */
    // 	eecd &= ~(E1000_EECD_DI | E1000_EECD_SK);
    // 	E1000_WRITE_REG(hw, E1000_EECD, eecd);
    // 	/* Set CS */
    // 	eecd |= E1000_EECD_CS;
    // 	E1000_WRITE_REG(hw, E1000_EECD, eecd);
    // } else if (nvm->type == e1000_nvm_eeprom_spi) {
    // 	u16 timeout = NVM_MAX_RETRY_SPI;

    // 	/* Clear SK and CS */
    // 	eecd &= ~(E1000_EECD_CS | E1000_EECD_SK);
    // 	E1000_WRITE_REG(hw, E1000_EECD, eecd);
    // 	E1000_WRITE_FLUSH(hw);
    // 	usec_delay(1);

    // 	/* Read "Status Register" repeatedly until the LSB is cleared.
    // 	 * The EEPROM will signal that the command has been completed
    // 	 * by clearing bit 0 of the internal status register.  If it's
    // 	 * not cleared within 'timeout', then error out.
    // 	 */
    // 	while (timeout) {
    // 		e1000_shift_out_eec_bits(hw, NVM_RDSR_OPCODE_SPI,
    // 					 hw->nvm.opcode_bits);
    // 		spi_stat_reg = (u8)e1000_shift_in_eec_bits(hw, 8);
    // 		if (!(spi_stat_reg & NVM_STATUS_RDY_SPI))
    // 			break;

    // 		usec_delay(5);
    // 		e1000_standby_nvm(hw);
    // 		timeout--;
    // 	}

    // 	if (!timeout) {
    // 		DEBUGOUT("SPI NVM Status error\n");
    // 		return -E1000_ERR_NVM;
    // 	}
    // }

    if adapter.hw.nvm.nvm_type == NvmType::EepromMicrowire {
        /* Clear SK and DI */
        // 	eecd &= ~(E1000_EECD_DI | E1000_EECD_SK);
        // 	E1000_WRITE_REG(hw, E1000_EECD, eecd);
        eecd &= !(E1000_EECD_DI | E1000_EECD_SK);
        do_write_register(adapter, E1000_EECD, eecd);

        /* Set CS */
        // 	eecd |= E1000_EECD_CS;
        // 	E1000_WRITE_REG(hw, E1000_EECD, eecd);
        eecd |= E1000_EECD_CS;
        do_write_register(adapter, E1000_EECD, eecd);

    } else if adapter.hw.nvm.nvm_type == NvmType::EepromSpi {
        incomplete!();
    }
    Ok(())
}

pub fn raise_eec_clk(adapter: &mut Adapter, eecd: &mut u32) {
    // e1000_println!();
    // *eecd = *eecd | E1000_EECD_SK;
    // E1000_WRITE_REG(hw, E1000_EECD, *eecd);
    // E1000_WRITE_FLUSH(hw);
    // usec_delay(hw->nvm.delay_usec);

    *eecd = *eecd | E1000_EECD_SK;
    do_write_register(adapter, E1000_EECD, *eecd);
    do_write_flush(adapter);
    do_usec_delay(adapter.hw.nvm.delay_usec as usize);
}

pub fn lower_eec_clk(adapter: &mut Adapter, eecd: &mut u32) {
    // e1000_println!();
    // *eecd = *eecd & ~E1000_EECD_SK;
    // E1000_WRITE_REG(hw, E1000_EECD, *eecd);
    // E1000_WRITE_FLUSH(hw);
    // usec_delay(hw->nvm.delay_usec);

    *eecd = *eecd & !E1000_EECD_SK;
    do_write_register(adapter, E1000_EECD, *eecd);
    do_write_flush(adapter);
    do_usec_delay(adapter.hw.nvm.delay_usec as usize);
}

pub fn shift_out_eec_bits(adapter: &mut Adapter, data: u16, count: u16) {
    // e1000_println!();
    // struct e1000_nvm_info *nvm = &hw->nvm;
    // u32 eecd = E1000_READ_REG(hw, E1000_EECD);
    // u32 mask;

    let mut eecd: u32 = do_read_register(adapter, E1000_EECD);

    // DEBUGFUNC("e1000_shift_out_eec_bits");

    // mask = 0x01 << (count - 1);
    // if (nvm->type == e1000_nvm_eeprom_microwire)
    // 	eecd &= ~E1000_EECD_DO;
    // else
    // if (nvm->type == e1000_nvm_eeprom_spi)
    // 	eecd |= E1000_EECD_DO;

    let mut mask: u32 = 0x01 << (count - 1);

    if adapter.hw.nvm.nvm_type == NvmType::EepromMicrowire {
        eecd &= !E1000_EECD_DO;
    } else if adapter.hw.nvm.nvm_type == NvmType::EepromSpi {
        incomplete!();
    }
    // do {
    // 	eecd &= ~E1000_EECD_DI;

    // 	if (data & mask)
    // 		eecd |= E1000_EECD_DI;

    // 	E1000_WRITE_REG(hw, E1000_EECD, eecd);
    // 	E1000_WRITE_FLUSH(hw);

    // 	usec_delay(nvm->delay_usec);

    // 	e1000_raise_eec_clk(hw, &eecd);
    // 	e1000_lower_eec_clk(hw, &eecd);

    // 	mask >>= 1;
    // } while (mask);

    loop {
        eecd &= !E1000_EECD_DI;

        if data & mask as u16 != 0 {
            eecd |= E1000_EECD_DI;
        }

        // e1000_println!("E_DI: {:032b}", E1000_EECD_DI);
        // e1000_println!("data: {:032b}", data);
        // e1000_println!("mask: {:032b}", mask);
        // e1000_println!("eecd: {:032b}", eecd);

        do_write_register(adapter, E1000_EECD, eecd);
        do_write_flush(adapter);

        do_usec_delay(adapter.hw.nvm.delay_usec as usize);

        raise_eec_clk(adapter, &mut eecd);
        lower_eec_clk(adapter, &mut eecd);

        mask >>= 1;

        if mask == 0 {
            break;
        }
    }

    // eecd &= ~E1000_EECD_DI;
    // E1000_WRITE_REG(hw, E1000_EECD, eecd);

    eecd &= !E1000_EECD_DI;
    do_write_register(adapter, E1000_EECD, eecd);
}

pub fn shift_in_eec_bits(adapter: &mut Adapter, count: u16) -> u16 {
    // e1000_println!();
    // u32 eecd;
    // u32 i;
    // u16 data;

    // DEBUGFUNC("e1000_shift_in_eec_bits");

    // eecd = E1000_READ_REG(hw, E1000_EECD);

    let mut eecd: u32 = do_read_register(adapter, E1000_EECD);

    // eecd &= ~(E1000_EECD_DO | E1000_EECD_DI);
    // data = 0;

    eecd &= !(E1000_EECD_DO | E1000_EECD_DI);
    let mut data: u16 = 0;


    // for (i = 0; i < count; i++) {
    // 	data <<= 1;
    // 	e1000_raise_eec_clk(hw, &eecd);

    // 	eecd = E1000_READ_REG(hw, E1000_EECD);

    // 	eecd &= ~E1000_EECD_DI;
    // 	if (eecd & E1000_EECD_DO)
    // 		data |= 1;

    // 	e1000_lower_eec_clk(hw, &eecd);
    // }

    for i in 0..count {
        data <<= 1;
        raise_eec_clk(adapter, &mut eecd);
        eecd = do_read_register(adapter, E1000_EECD);
        eecd &= !E1000_EECD_DI;
        // println!("read bits 0x{:x}", eecd);
        if eecd & E1000_EECD_DO != 0 {
            data |= 1;
        }

        // e1000_println!("E_DO: {:032b}", E1000_EECD_DO);
        // e1000_println!("data: {:032b}", data);
        // e1000_println!("eecd: {:032b}", eecd);

        lower_eec_clk(adapter, &mut eecd);
    }
    data
}

pub fn stop_nvm(adapter: &mut Adapter) {
    // e1000_println!();
    let mut eecd;

    // eecd = E1000_READ_REG(hw, E1000_EECD);

    // if (hw->nvm.type == e1000_nvm_eeprom_spi) {
    // 	/* Pull CS high */
    // eecd |= E1000_EECD_CS;
    // e1000_lower_eec_clk(hw, &eecd);
    // } else if (hw->nvm.type == e1000_nvm_eeprom_microwire) {
    // 	/* CS on Microwire is active-high */
    // 	eecd &= ~(E1000_EECD_CS | E1000_EECD_DI);
    // 	E1000_WRITE_REG(hw, E1000_EECD, eecd);
    // 	e1000_raise_eec_clk(hw, &eecd);
    // 	e1000_lower_eec_clk(hw, &eecd);
    // }

    eecd = do_read_register(adapter, E1000_EECD);
    if adapter.hw.nvm.nvm_type == NvmType::EepromSpi {
        /* Pull CS high */
        eecd |= E1000_EECD_CS;
        lower_eec_clk(adapter, &mut eecd);
    } else if adapter.hw.nvm.nvm_type == NvmType::EepromMicrowire {
        /* CS on Microwire is active-high */
        eecd &= !(E1000_EECD_CS | E1000_EECD_DI);
        do_write_register(adapter, E1000_EECD, eecd);
        raise_eec_clk(adapter, &mut eecd);
        lower_eec_clk(adapter, &mut eecd);
    }
}

pub fn standby_nvm(adapter: &mut Adapter) {
    // e1000_println!();

    // u32 eecd = E1000_READ_REG(hw, E1000_EECD);
    let mut eecd: u32 = do_read_register(adapter, E1000_EECD);

    // if (nvm->type == e1000_nvm_eeprom_microwire) {
    if adapter.hw.nvm.nvm_type == NvmType::EepromMicrowire {
        // 	eecd &= ~(E1000_EECD_CS | E1000_EECD_SK);
        // 	E1000_WRITE_REG(hw, E1000_EECD, eecd);
        // 	E1000_WRITE_FLUSH(hw);
        // 	usec_delay(nvm->delay_usec);
        // 	e1000_raise_eec_clk(hw, &eecd);

        // 	eecd |= E1000_EECD_CS;
        // 	E1000_WRITE_REG(hw, E1000_EECD, eecd);
        // 	E1000_WRITE_FLUSH(hw);
        // 	usec_delay(nvm->delay_usec);
        // 	e1000_lower_eec_clk(hw, &eecd);

        eecd &= !(E1000_EECD_CS | E1000_EECD_SK);
        do_write_register(adapter, E1000_EECD, eecd);
        do_write_flush(adapter);
        do_usec_delay(adapter.hw.nvm.delay_usec as usize);
        raise_eec_clk(adapter, &mut eecd);

        /* Select EEPROM */
        eecd |= E1000_EECD_CS;
        do_write_register(adapter, E1000_EECD, eecd);
        do_write_flush(adapter);
        do_usec_delay(adapter.hw.nvm.delay_usec as usize);
        lower_eec_clk(adapter, &mut eecd);

    } else if adapter.hw.nvm.nvm_type == NvmType::EepromSpi {
        // } else if (nvm->type == e1000_nvm_eeprom_spi) {
        // 	/* Toggle CS to flush commands */
        // 	eecd |= E1000_EECD_CS;
        // 	E1000_WRITE_REG(hw, E1000_EECD, eecd);
        // 	E1000_WRITE_FLUSH(hw);
        // 	usec_delay(nvm->delay_usec);
        // 	eecd &= ~E1000_EECD_CS;
        // 	E1000_WRITE_REG(hw, E1000_EECD, eecd);
        // 	E1000_WRITE_FLUSH(hw);
        // 	usec_delay(nvm->delay_usec);
        // }
        incomplete!();
    }
}
