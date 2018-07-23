
pub fn E1000_RAL(i: usize) -> u32 {
    let i = i as u32;
    match i {
        x if x <= 15 => 0x05400 + i * 8,
        _ => 0x054E0 + (i - 16) * 8,
    }
}

pub fn E1000_RAH(i: usize) -> u32 {
    let i = i as u32;
    match i {
        x if x <= 15 => 0x05404 + i * 8,
        _ => 0x054E4 + (i - 16) * 8,
    }
}

pub fn E1000_TDBAL(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x03800 + n * 0x100,
        _ => 0x0E000 + n * 0x40,
    }
}

pub fn E1000_TDBAH(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x03804 + n * 0x100,
        _ => 0x0E004 + n * 0x40,
    }
}

pub fn E1000_TDLEN(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x03808 + n * 0x100,
        _ => 0x0E008 + n * 0x40,
    }
}

pub fn E1000_TDH(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x03810 + n * 0x100,
        _ => 0x0E010 + n * 0x40,
    }
}

pub fn E1000_TXCTL(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x03814 + n * 0x100,
        _ => 0x0E014 + n * 0x40,
    }
}

pub fn E1000_TDT(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x03818 + n * 0x100,
        _ => 0x0E018 + n * 0x40,
    }
}

pub fn E1000_TXDCTL(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x03828 + n * 0x100,
        _ => 0x0E028 + n * 0x40,
    }
}



pub fn E1000_RDBAL(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x02800 + n * 0x100,
        _ => 0x0C000 + n * 0x40,
    }
}

pub fn E1000_RDBAH(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x02804 + n * 0x100,
        _ => 0x0C004 + n * 0x40,
    }
}

pub fn E1000_RDLEN(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x02808 + n * 0x100,
        _ => 0x0C008 + n * 0x40,
    }
}

pub fn E1000_RDH(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x02810 + n * 0x100,
        _ => 0x0C010 + n * 0x40,
    }
}

pub fn E1000_RXCTL(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x02814 + n * 0x100,
        _ => 0x0C014 + n * 0x40,
    }
}

pub fn E1000_RDT(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x02818 + n * 0x100,
        _ => 0x0C018 + n * 0x40,
    }
}

pub fn E1000_RXDCTL(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x02828 + n * 0x100,
        _ => 0x0C028 + n * 0x40,
    }
}

pub fn E1000_TARC(n: usize) -> u32 {
    0x3840 + n as u32 * 0x100
}

use sys::e1000_consts::MAX_PHY_REG_ADDRESS;
use sys::e1000_consts::PHY_PAGE_SHIFT;
use sys::e1000_consts::PHY_UPPER_SHIFT;
use sys::e1000_consts::BM_WUC_PAGE;
pub const fn BM_PHY_REG(page: u16, reg: u32) -> u32 {
    ((reg & MAX_PHY_REG_ADDRESS) | ((page as u32) << PHY_PAGE_SHIFT) |
         (reg & !MAX_PHY_REG_ADDRESS) << (PHY_UPPER_SHIFT - PHY_PAGE_SHIFT))
}

pub fn BM_PHY_REG_PAGE(offset: u32) -> u16 {
    ((offset >> PHY_PAGE_SHIFT) & 0xFFFF) as u16
}

pub fn BM_PHY_REG_NUM(offset: u32) -> u16 {
    ((offset & MAX_PHY_REG_ADDRESS) |
         (offset >> (PHY_UPPER_SHIFT - PHY_PAGE_SHIFT)) & !MAX_PHY_REG_ADDRESS) as u16
}

pub const fn BM_MTA(i: usize) -> u32 {
    BM_PHY_REG(BM_WUC_PAGE, 128 + ((i as u32) << 1))
}

/* Shared Receive Address Registers */
pub fn E1000_SHRAL_PCH_LPT(i: usize) -> u32 {
    let i = i as u32;
    0x05408 + (i * 8)
}

pub fn E1000_SHRAH_PCH_LPT(i: usize) -> u32 {
    let i = i as u32;
    0x0540C + (i * 8)
}
