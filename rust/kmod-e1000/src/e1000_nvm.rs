
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

    let mut eecd: u32 = do_read_register(adapter, E1000_EECD);
    let mut timeout: i32 = E1000_NVM_GRANT_ATTEMPTS as i32;

    do_write_register(adapter, E1000_EECD, eecd | E1000_EECD_REQ);
    eecd = do_read_register(adapter, E1000_EECD);

    while timeout > 0 {
        if eecd & E1000_EECD_GNT > 0 {
            break;
        }
        do_usec_delay(5);
        eecd = do_read_register(adapter, E1000_EECD);
        timeout -= 1;
    }

    if timeout == 0 {
        eecd &= !E1000_EECD_REQ;
        do_write_register(adapter, E1000_EECD, eecd);
        Err("Could not acquire NVM grant".to_string())
    } else {
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

    let read_opcode: u8 = NVM_READ_OPCODE_MICROWIRE as u8;

    /* A check for invalid values:  offset too large, too many words,
     * and not enough words.
     */
    let word_size = adapter.hw.nvm.word_size;
    if offset >= word_size || words > (word_size - offset) || words == 0 {
        return Err("nvm parameter out of bounds".into());
    }

    let acquire = try!(adapter.hw.nvm.ops.acquire.ok_or("No function".to_string()));
    let release = try!(adapter.hw.nvm.ops.release.ok_or("No function".to_string()));

    try!(acquire(adapter));

    let _ = ready_nvm_eeprom(adapter).or_else(|e| {
        release(adapter);
        return Err(e);
    });

    let opcode_bits = adapter.hw.nvm.opcode_bits;
    let address_bits = adapter.hw.nvm.address_bits;

    for i in 0..words as usize {
        /* Send the READ command (opcode + addr) */
        shift_out_eec_bits(adapter, read_opcode as u16, opcode_bits);
        shift_out_eec_bits(adapter, offset + i as u16, address_bits);
        /* Read the data.  For microwire, each word requires the
         * overhead of setup and tear-down.
         */
        let value: u16 = shift_in_eec_bits(adapter, 16);
        data[i] = value;
        standby_nvm(adapter);
    }
    release(adapter);
    Ok(())
}

pub fn release_nvm_generic(adapter: &mut Adapter) {
    // e1000_println!();

    let mut eecd: u32;
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

    let mut nvm_data: [u16; 1] = [0u16; 1];
    let mut checksum: u16 = 0;

    let read = try!(adapter.hw.nvm.ops.read.ok_or("No function".to_string()));

    for i in 0..(NVM_CHECKSUM_REG + 1) as u16 {
        try!(read(adapter, i, 1, &mut nvm_data));
        checksum += nvm_data[0];
    }

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

    let mut eecd: u32 = do_read_register(adapter, E1000_EECD);

    if adapter.hw.nvm.nvm_type == NvmType::EepromMicrowire {
        /* Clear SK and DI */
        eecd &= !(E1000_EECD_DI | E1000_EECD_SK);
        do_write_register(adapter, E1000_EECD, eecd);

        /* Set CS */
        eecd |= E1000_EECD_CS;
        do_write_register(adapter, E1000_EECD, eecd);

    } else if adapter.hw.nvm.nvm_type == NvmType::EepromSpi {
        incomplete!();
    }
    Ok(())
}

pub fn raise_eec_clk(adapter: &mut Adapter, eecd: &mut u32) {
    // e1000_println!();

    *eecd = *eecd | E1000_EECD_SK;
    do_write_register(adapter, E1000_EECD, *eecd);
    do_write_flush(adapter);
    do_usec_delay(adapter.hw.nvm.delay_usec as usize);
}

pub fn lower_eec_clk(adapter: &mut Adapter, eecd: &mut u32) {
    // e1000_println!();

    *eecd = *eecd & !E1000_EECD_SK;
    do_write_register(adapter, E1000_EECD, *eecd);
    do_write_flush(adapter);
    do_usec_delay(adapter.hw.nvm.delay_usec as usize);
}

pub fn shift_out_eec_bits(adapter: &mut Adapter, data: u16, count: u16) {
    // e1000_println!();

    let mut eecd: u32 = do_read_register(adapter, E1000_EECD);
    let mut mask: u32 = 0x01 << (count - 1);

    if adapter.hw.nvm.nvm_type == NvmType::EepromMicrowire {
        eecd &= !E1000_EECD_DO;
    } else if adapter.hw.nvm.nvm_type == NvmType::EepromSpi {
        incomplete!();
    }

    loop {
        eecd &= !E1000_EECD_DI;

        if data & mask as u16 != 0 {
            eecd |= E1000_EECD_DI;
        }

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

    eecd &= !E1000_EECD_DI;
    do_write_register(adapter, E1000_EECD, eecd);
}

pub fn shift_in_eec_bits(adapter: &mut Adapter, count: u16) -> u16 {
    // e1000_println!();

    let mut eecd: u32 = do_read_register(adapter, E1000_EECD);
    eecd &= !(E1000_EECD_DO | E1000_EECD_DI);
    let mut data: u16 = 0;

    for i in 0..count {
        data <<= 1;
        raise_eec_clk(adapter, &mut eecd);
        eecd = do_read_register(adapter, E1000_EECD);
        eecd &= !E1000_EECD_DI;
        if eecd & E1000_EECD_DO != 0 {
            data |= 1;
        }
        lower_eec_clk(adapter, &mut eecd);
    }
    data
}

pub fn stop_nvm(adapter: &mut Adapter) {
    // e1000_println!();

    let mut eecd;

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

    let mut eecd: u32 = do_read_register(adapter, E1000_EECD);

    if adapter.hw.nvm.nvm_type == NvmType::EepromMicrowire {
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
        incomplete!();
    }
}
