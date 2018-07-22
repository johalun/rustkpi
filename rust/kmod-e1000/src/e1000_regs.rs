

// #define E1000_RAL(_i)		(((_i) <= 15) ? (0x05400 + ((_i) * 8)) : \
// 				 (0x054E0 + ((_i - 16) * 8)))
pub fn E1000_RAL(i: usize) -> u32 {
    let i = i as u32;
    match i {
        x if x <= 15 => 0x05400 + i * 8,
        _ => 0x054E0 + (i - 16) * 8,
    }
}

// #define E1000_RAH(_i)		(((_i) <= 15) ? (0x05404 + ((_i) * 8)) : \
// 				 (0x054E4 + ((_i - 16) * 8)))
pub fn E1000_RAH(i: usize) -> u32 {
    let i = i as u32;
    match i {
        x if x <= 15 => 0x05404 + i * 8,
        _ => 0x054E4 + (i - 16) * 8,
    }
}

// #define E1000_TDBAL(_n)	((_n) < 4 ? (0x03800 + ((_n) * 0x100)) : \
// 			 (0x0E000 + ((_n) * 0x40)))
pub fn E1000_TDBAL(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x03800 + n * 0x100,
        _ => 0x0E000 + n * 0x40,
    }
}

// #define E1000_TDBAH(_n)	((_n) < 4 ? (0x03804 + ((_n) * 0x100)) : \
// 			 (0x0E004 + ((_n) * 0x40)))
pub fn E1000_TDBAH(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x03804 + n * 0x100,
        _ => 0x0E004 + n * 0x40,
    }
}

// #define E1000_TDLEN(_n)	((_n) < 4 ? (0x03808 + ((_n) * 0x100)) : \
// 			 (0x0E008 + ((_n) * 0x40)))
pub fn E1000_TDLEN(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x03808 + n * 0x100,
        _ => 0x0E008 + n * 0x40,
    }
}

// #define E1000_TDH(_n)	((_n) < 4 ? (0x03810 + ((_n) * 0x100)) : \
// 			 (0x0E010 + ((_n) * 0x40)))
pub fn E1000_TDH(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x03810 + n * 0x100,
        _ => 0x0E010 + n * 0x40,
    }
}

// #define E1000_TXCTL(_n)	((_n) < 4 ? (0x03814 + ((_n) * 0x100)) : \
// 			 (0x0E014 + ((_n) * 0x40)))
pub fn E1000_TXCTL(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x03814 + n * 0x100,
        _ => 0x0E014 + n * 0x40,
    }
}

// #define E1000_TDT(_n)	((_n) < 4 ? (0x03818 + ((_n) * 0x100)) : \
// 			 (0x0E018 + ((_n) * 0x40)))
pub fn E1000_TDT(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x03818 + n * 0x100,
        _ => 0x0E018 + n * 0x40,
    }
}

// #define E1000_TXDCTL(_n)	((_n) < 4 ? (0x03828 + ((_n) * 0x100)) : \
// 				 (0x0E028 + ((_n) * 0x40)))
pub fn E1000_TXDCTL(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x03828 + n * 0x100,
        _ => 0x0E028 + n * 0x40,
    }
}




// #define E1000_RDBAL(_n)	((_n) < 4 ? (0x02800 + ((_n) * 0x100)) : \
// 			 (0x0C000 + ((_n) * 0x40)))
pub fn E1000_RDBAL(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x02800 + n * 0x100,
        _ => 0x0C000 + n * 0x40,
    }
}

// #define E1000_RDBAH(_n)	((_n) < 4 ? (0x02804 + ((_n) * 0x100)) : \
// (0x0C004 + ((_n) * 0x40)))
pub fn E1000_RDBAH(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x02804 + n * 0x100,
        _ => 0x0C004 + n * 0x40,
    }
}

// #define E1000_RDLEN(_n)	((_n) < 4 ? (0x02808 + ((_n) * 0x100)) : \
// (0x0C008 + ((_n) * 0x40)))
pub fn E1000_RDLEN(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x02808 + n * 0x100,
        _ => 0x0C008 + n * 0x40,
    }
}

// #define E1000_RDH(_n)	((_n) < 4 ? (0x02810 + ((_n) * 0x100)) : \
// (0x0C010 + ((_n) * 0x40)))
pub fn E1000_RDH(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x02810 + n * 0x100,
        _ => 0x0C010 + n * 0x40,
    }
}

// #define E1000_RXCTL(_n)	((_n) < 4 ? (0x02814 + ((_n) * 0x100)) : \
// (0x0C014 + ((_n) * 0x40)))
pub fn E1000_RXCTL(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x02814 + n * 0x100,
        _ => 0x0C014 + n * 0x40,
    }
}

// #define E1000_RDT(_n)	((_n) < 4 ? (0x02818 + ((_n) * 0x100)) : \
// (0x0C018 + ((_n) * 0x40)))
pub fn E1000_RDT(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x02818 + n * 0x100,
        _ => 0x0C018 + n * 0x40,
    }
}

// #define E1000_RXDCTL(_n)	((_n) < 4 ? (0x02828 + ((_n) * 0x100)) : \
// (0x0C028 + ((_n) * 0x40)))
pub fn E1000_RXDCTL(n: usize) -> u32 {
    let n = n as u32;
    match n {
        x if x < 4 => 0x02828 + n * 0x100,
        _ => 0x0C028 + n * 0x40,
    }
}

// #define E1000_TARC(_n)		(0x03840 + ((_n) * 0x100))
pub fn E1000_TARC(n: usize) -> u32 {
    0x3840 + n as u32 * 0x100
}

// #define BM_PHY_REG(page, reg) \
// 	(((reg) & MAX_PHY_REG_ADDRESS) |\
// 	 (((page) & 0xFFFF) << PHY_PAGE_SHIFT) |\
// 	 (((reg) & ~MAX_PHY_REG_ADDRESS) << (PHY_UPPER_SHIFT - PHY_PAGE_SHIFT)))
use sys::e1000_consts::MAX_PHY_REG_ADDRESS;
use sys::e1000_consts::PHY_PAGE_SHIFT;
use sys::e1000_consts::PHY_UPPER_SHIFT;
use sys::e1000_consts::BM_WUC_PAGE;
pub const fn BM_PHY_REG(page: u16, reg: u32) -> u32 {
    ((reg & MAX_PHY_REG_ADDRESS) | ((page as u32) << PHY_PAGE_SHIFT) |
         (reg & !MAX_PHY_REG_ADDRESS) << (PHY_UPPER_SHIFT - PHY_PAGE_SHIFT))
}

// #define BM_PHY_REG_PAGE(offset) \
// 	((u16)(((offset) >> PHY_PAGE_SHIFT) & 0xFFFF))
pub fn BM_PHY_REG_PAGE(offset: u32) -> u16 {
    ((offset >> PHY_PAGE_SHIFT) & 0xFFFF) as u16
}

// #define BM_PHY_REG_NUM(offset) \
// 	((u16)(((offset) & MAX_PHY_REG_ADDRESS) |\
// 	 (((offset) >> (PHY_UPPER_SHIFT - PHY_PAGE_SHIFT)) & ~MAX_PHY_REG_ADDRESS)
//	))
pub fn BM_PHY_REG_NUM(offset: u32) -> u16 {
    ((offset & MAX_PHY_REG_ADDRESS) |
         (offset >> (PHY_UPPER_SHIFT - PHY_PAGE_SHIFT)) & !MAX_PHY_REG_ADDRESS) as u16
}

// #define BM_MTA(_i)		(BM_PHY_REG(BM_WUC_PAGE, 128 + ((_i) << 1)))
pub const fn BM_MTA(i: usize) -> u32 {
    BM_PHY_REG(BM_WUC_PAGE, 128 + ((i as u32) << 1))
}

/* Shared Receive Address Registers */
// #define E1000_SHRAL_PCH_LPT(_i)		(0x05408 + ((_i) * 8))
pub fn E1000_SHRAL_PCH_LPT(i: usize) -> u32 {
    let i = i as u32;
    0x05408 + (i * 8)
}

// #define E1000_SHRAH_PCH_LPT(_i)		(0x0540C + ((_i) * 8))
pub fn E1000_SHRAH_PCH_LPT(i: usize) -> u32 {
    let i = i as u32;
    0x0540C + (i * 8)
}


// pub const E1000_CTRL: ::kernel::sys::raw::c_uint = 0;
// pub const E1000_CTRL_DUP: ::kernel::sys::raw::c_uint = 4;
// pub const E1000_STATUS: ::kernel::sys::raw::c_uint = 8;
// pub const E1000_EECD: ::kernel::sys::raw::c_uint = 16;
// pub const E1000_EERD: ::kernel::sys::raw::c_uint = 20;
// pub const E1000_CTRL_EXT: ::kernel::sys::raw::c_uint = 24;
// pub const E1000_FLA: ::kernel::sys::raw::c_uint = 28;
// pub const E1000_MDIC: ::kernel::sys::raw::c_uint = 32;
// pub const E1000_MDICNFG: ::kernel::sys::raw::c_uint = 3588;
// pub const E1000_REGISTER_SET_SIZE: ::kernel::sys::raw::c_uint = 131072;
// pub const E1000_EEPROM_INIT_CTRL_WORD_2: ::kernel::sys::raw::c_uint = 15;
// pub const E1000_EEPROM_PCIE_CTRL_WORD_2: ::kernel::sys::raw::c_uint = 40;
// pub const E1000_BARCTRL: ::kernel::sys::raw::c_uint = 23484;
// pub const E1000_BARCTRL_FLSIZE: ::kernel::sys::raw::c_uint = 1792;
// pub const E1000_BARCTRL_CSRSIZE: ::kernel::sys::raw::c_uint = 8192;
// pub const E1000_MPHY_ADDR_CTRL: ::kernel::sys::raw::c_uint = 36;
// pub const E1000_MPHY_DATA: ::kernel::sys::raw::c_uint = 3600;
// pub const E1000_MPHY_STAT: ::kernel::sys::raw::c_uint = 3596;
// pub const E1000_PPHY_CTRL: ::kernel::sys::raw::c_uint = 23368;
// pub const E1000_I350_BARCTRL: ::kernel::sys::raw::c_uint = 23548;
// pub const E1000_I350_DTXMXPKTSZ: ::kernel::sys::raw::c_uint = 13660;
// pub const E1000_SCTL: ::kernel::sys::raw::c_uint = 36;
// pub const E1000_FCAL: ::kernel::sys::raw::c_uint = 40;
// pub const E1000_FCAH: ::kernel::sys::raw::c_uint = 44;
// pub const E1000_FEXT: ::kernel::sys::raw::c_uint = 44;
// pub const E1000_FEXTNVM: ::kernel::sys::raw::c_uint = 40;
// pub const E1000_FEXTNVM3: ::kernel::sys::raw::c_uint = 60;
// pub const E1000_FEXTNVM4: ::kernel::sys::raw::c_uint = 36;
// pub const E1000_FEXTNVM6: ::kernel::sys::raw::c_uint = 16;
// pub const E1000_FEXTNVM7: ::kernel::sys::raw::c_uint = 228;
// pub const E1000_FEXTNVM9: ::kernel::sys::raw::c_uint = 23476;
// pub const E1000_FEXTNVM11: ::kernel::sys::raw::c_uint = 23484;
// pub const E1000_PCIEANACFG: ::kernel::sys::raw::c_uint = 3864;
// pub const E1000_FCT: ::kernel::sys::raw::c_uint = 48;
// pub const E1000_CONNSW: ::kernel::sys::raw::c_uint = 52;
// pub const E1000_VET: ::kernel::sys::raw::c_uint = 56;
// pub const E1000_ICR: ::kernel::sys::raw::c_uint = 192;
// pub const E1000_ITR: ::kernel::sys::raw::c_uint = 196;
// pub const E1000_ICS: ::kernel::sys::raw::c_uint = 200;
// pub const E1000_IMS: ::kernel::sys::raw::c_uint = 208;
// pub const E1000_IMC: ::kernel::sys::raw::c_uint = 216;
// pub const E1000_IAM: ::kernel::sys::raw::c_uint = 224;
// pub const E1000_IVAR: ::kernel::sys::raw::c_uint = 228;
// pub const E1000_SVCR: ::kernel::sys::raw::c_uint = 240;
// pub const E1000_SVT: ::kernel::sys::raw::c_uint = 244;
// pub const E1000_LPIC: ::kernel::sys::raw::c_uint = 252;
// pub const E1000_RCTL: ::kernel::sys::raw::c_uint = 256;
// pub const E1000_FCTTV: ::kernel::sys::raw::c_uint = 368;
// pub const E1000_TXCW: ::kernel::sys::raw::c_uint = 376;
// pub const E1000_RXCW: ::kernel::sys::raw::c_uint = 384;
// pub const E1000_PBA_ECC: ::kernel::sys::raw::c_uint = 4352;
// pub const E1000_EICR: ::kernel::sys::raw::c_uint = 5504;
// pub const E1000_EICS: ::kernel::sys::raw::c_uint = 5408;
// pub const E1000_EIMS: ::kernel::sys::raw::c_uint = 5412;
// pub const E1000_EIMC: ::kernel::sys::raw::c_uint = 5416;
// pub const E1000_EIAC: ::kernel::sys::raw::c_uint = 5420;
// pub const E1000_EIAM: ::kernel::sys::raw::c_uint = 5424;
// pub const E1000_GPIE: ::kernel::sys::raw::c_uint = 5396;
// pub const E1000_IVAR0: ::kernel::sys::raw::c_uint = 5888;
// pub const E1000_IVAR_MISC: ::kernel::sys::raw::c_uint = 5952;
// pub const E1000_TCTL: ::kernel::sys::raw::c_uint = 1024;
// pub const E1000_TCTL_EXT: ::kernel::sys::raw::c_uint = 1028;
// pub const E1000_TIPG: ::kernel::sys::raw::c_uint = 1040;
// pub const E1000_TBT: ::kernel::sys::raw::c_uint = 1096;
// pub const E1000_AIT: ::kernel::sys::raw::c_uint = 1112;
// pub const E1000_LEDCTL: ::kernel::sys::raw::c_uint = 3584;
// pub const E1000_LEDMUX: ::kernel::sys::raw::c_uint = 33072;
// pub const E1000_EXTCNF_CTRL: ::kernel::sys::raw::c_uint = 3840;
// pub const E1000_EXTCNF_SIZE: ::kernel::sys::raw::c_uint = 3848;
// pub const E1000_PHY_CTRL: ::kernel::sys::raw::c_uint = 3856;
// pub const E1000_POEMB: ::kernel::sys::raw::c_uint = 3856;
// pub const E1000_PBA: ::kernel::sys::raw::c_uint = 4096;
// pub const E1000_PBS: ::kernel::sys::raw::c_uint = 4104;
// pub const E1000_PBECCSTS: ::kernel::sys::raw::c_uint = 4108;
// pub const E1000_IOSFPC: ::kernel::sys::raw::c_uint = 3880;
// pub const E1000_EEMNGCTL: ::kernel::sys::raw::c_uint = 4112;
// pub const E1000_EEMNGCTL_I210: ::kernel::sys::raw::c_uint = 4112;
// pub const E1000_EEARBC: ::kernel::sys::raw::c_uint = 4132;
// pub const E1000_EEARBC_I210: ::kernel::sys::raw::c_uint = 73764;
// pub const E1000_FLASHT: ::kernel::sys::raw::c_uint = 4136;
// pub const E1000_EEWR: ::kernel::sys::raw::c_uint = 4140;
// pub const E1000_FLSWCTL: ::kernel::sys::raw::c_uint = 4144;
// pub const E1000_FLSWDATA: ::kernel::sys::raw::c_uint = 4148;
// pub const E1000_FLSWCNT: ::kernel::sys::raw::c_uint = 4152;
// pub const E1000_FLOP: ::kernel::sys::raw::c_uint = 4156;
// pub const E1000_I2CCMD: ::kernel::sys::raw::c_uint = 4136;
// pub const E1000_I2CPARAMS: ::kernel::sys::raw::c_uint = 4140;
// pub const E1000_I2CBB_EN: ::kernel::sys::raw::c_uint = 256;
// pub const E1000_I2C_CLK_OUT: ::kernel::sys::raw::c_uint = 512;
// pub const E1000_I2C_DATA_OUT: ::kernel::sys::raw::c_uint = 1024;
// pub const E1000_I2C_DATA_OE_N: ::kernel::sys::raw::c_uint = 2048;
// pub const E1000_I2C_DATA_IN: ::kernel::sys::raw::c_uint = 4096;
// pub const E1000_I2C_CLK_OE_N: ::kernel::sys::raw::c_uint = 8192;
// pub const E1000_I2C_CLK_IN: ::kernel::sys::raw::c_uint = 16384;
// pub const E1000_I2C_CLK_STRETCH_DIS: ::kernel::sys::raw::c_uint = 32768;
// pub const E1000_WDSTP: ::kernel::sys::raw::c_uint = 4160;
// pub const E1000_SWDSTS: ::kernel::sys::raw::c_uint = 4164;
// pub const E1000_FRTIMER: ::kernel::sys::raw::c_uint = 4168;
// pub const E1000_TCPTIMER: ::kernel::sys::raw::c_uint = 4172;
// pub const E1000_VPDDIAG: ::kernel::sys::raw::c_uint = 4192;
// pub const E1000_ICR_V2: ::kernel::sys::raw::c_uint = 5376;
// pub const E1000_ICS_V2: ::kernel::sys::raw::c_uint = 5380;
// pub const E1000_IMS_V2: ::kernel::sys::raw::c_uint = 5384;
// pub const E1000_IMC_V2: ::kernel::sys::raw::c_uint = 5388;
// pub const E1000_IAM_V2: ::kernel::sys::raw::c_uint = 5392;
// pub const E1000_ERT: ::kernel::sys::raw::c_uint = 8200;
// pub const E1000_FCRTL: ::kernel::sys::raw::c_uint = 8544;
// pub const E1000_FCRTH: ::kernel::sys::raw::c_uint = 8552;
// pub const E1000_PSRCTL: ::kernel::sys::raw::c_uint = 8560;
// pub const E1000_RDFH: ::kernel::sys::raw::c_uint = 9232;
// pub const E1000_RDFT: ::kernel::sys::raw::c_uint = 9240;
// pub const E1000_RDFHS: ::kernel::sys::raw::c_uint = 9248;
// pub const E1000_RDFTS: ::kernel::sys::raw::c_uint = 9256;
// pub const E1000_RDFPC: ::kernel::sys::raw::c_uint = 9264;
// pub const E1000_PBRTH: ::kernel::sys::raw::c_uint = 9304;
// pub const E1000_FCRTV: ::kernel::sys::raw::c_uint = 9312;
// pub const E1000_RDPUMB: ::kernel::sys::raw::c_uint = 9676;
// pub const E1000_RDPUAD: ::kernel::sys::raw::c_uint = 9680;
// pub const E1000_RDPUWD: ::kernel::sys::raw::c_uint = 9684;
// pub const E1000_RDPURD: ::kernel::sys::raw::c_uint = 9688;
// pub const E1000_RDPUCTL: ::kernel::sys::raw::c_uint = 9692;
// pub const E1000_PBDIAG: ::kernel::sys::raw::c_uint = 9304;
// pub const E1000_RXPBS: ::kernel::sys::raw::c_uint = 9220;
// pub const E1000_IRPBS: ::kernel::sys::raw::c_uint = 9220;
// pub const E1000_PBRWAC: ::kernel::sys::raw::c_uint = 9448;
// pub const E1000_RDTR: ::kernel::sys::raw::c_uint = 10272;
// pub const E1000_RADV: ::kernel::sys::raw::c_uint = 10284;
// pub const E1000_EMIADD: ::kernel::sys::raw::c_uint = 16;
// pub const E1000_EMIDATA: ::kernel::sys::raw::c_uint = 17;
// pub const E1000_SRWR: ::kernel::sys::raw::c_uint = 73752;
// pub const E1000_I210_FLMNGCTL: ::kernel::sys::raw::c_uint = 73784;
// pub const E1000_I210_FLMNGDATA: ::kernel::sys::raw::c_uint = 73788;
// pub const E1000_I210_FLMNGCNT: ::kernel::sys::raw::c_uint = 73792;
// pub const E1000_I210_FLSWCTL: ::kernel::sys::raw::c_uint = 73800;
// pub const E1000_I210_FLSWDATA: ::kernel::sys::raw::c_uint = 73804;
// pub const E1000_I210_FLSWCNT: ::kernel::sys::raw::c_uint = 73808;
// pub const E1000_I210_FLA: ::kernel::sys::raw::c_uint = 73756;
// pub const E1000_INVM_SIZE: ::kernel::sys::raw::c_uint = 64;
// pub const E1000_I210_TQAVCTRL: ::kernel::sys::raw::c_uint = 13680;
// pub const E1000_TQAVCTRL_MODE: ::kernel::sys::raw::c_uint = 1;
// pub const E1000_TQAVCTRL_FETCH_ARB: ::kernel::sys::raw::c_uint = 16;
// pub const E1000_TQAVCTRL_FETCH_TIMER_ENABLE: ::kernel::sys::raw::c_uint = 32;
// pub const E1000_TQAVCTRL_LAUNCH_ARB: ::kernel::sys::raw::c_uint = 256;
// pub const E1000_TQAVCTRL_LAUNCH_TIMER_ENABLE: ::kernel::sys::raw::c_uint = 512;
// pub const E1000_TQAVCTRL_SP_WAIT_SR: ::kernel::sys::raw::c_uint = 1024;
// pub const E1000_TQAVCTRL_FETCH_TIMER_DELTA_OFFSET: ::kernel::sys::raw::c_uint = 16;
// pub const E1000_TQAVCTRL_FETCH_TIMER_DELTA: ::kernel::sys::raw::c_uint = 4294901760;
// pub const E1000_I210_TQAVARBCTRL: ::kernel::sys::raw::c_uint = 13684;
// pub const E1000_TQAVCC_IDLE_SLOPE: ::kernel::sys::raw::c_uint = 65535;
// pub const E1000_TQAVCC_KEEP_CREDITS: ::kernel::sys::raw::c_uint = 1073741824;
// pub const E1000_TQAVCC_QUEUE_MODE: ::kernel::sys::raw::c_uint = 2147483648;
// pub const E1000_MMDAC: ::kernel::sys::raw::c_uint = 13;
// pub const E1000_MMDAAD: ::kernel::sys::raw::c_uint = 14;
// pub const E1000_RSRPD: ::kernel::sys::raw::c_uint = 11264;
// pub const E1000_RAID: ::kernel::sys::raw::c_uint = 11272;
// pub const E1000_TXDMAC: ::kernel::sys::raw::c_uint = 12288;
// pub const E1000_KABGTXD: ::kernel::sys::raw::c_uint = 12292;
// pub const E1000_PBSLAC: ::kernel::sys::raw::c_uint = 12544;
// pub const E1000_TXPBS: ::kernel::sys::raw::c_uint = 13316;
// pub const E1000_ITPBS: ::kernel::sys::raw::c_uint = 13316;
// pub const E1000_TDFH: ::kernel::sys::raw::c_uint = 13328;
// pub const E1000_TDFT: ::kernel::sys::raw::c_uint = 13336;
// pub const E1000_TDFHS: ::kernel::sys::raw::c_uint = 13344;
// pub const E1000_TDFTS: ::kernel::sys::raw::c_uint = 13352;
// pub const E1000_TDFPC: ::kernel::sys::raw::c_uint = 13360;
// pub const E1000_TDPUMB: ::kernel::sys::raw::c_uint = 13692;
// pub const E1000_TDPUAD: ::kernel::sys::raw::c_uint = 13696;
// pub const E1000_TDPUWD: ::kernel::sys::raw::c_uint = 13700;
// pub const E1000_TDPURD: ::kernel::sys::raw::c_uint = 13704;
// pub const E1000_TDPUCTL: ::kernel::sys::raw::c_uint = 13708;
// pub const E1000_DTXCTL: ::kernel::sys::raw::c_uint = 13712;
// pub const E1000_DTXTCPFLGL: ::kernel::sys::raw::c_uint = 13724;
// pub const E1000_DTXTCPFLGH: ::kernel::sys::raw::c_uint = 13728;
// pub const E1000_DTXMXSZRQ: ::kernel::sys::raw::c_uint = 13632;
// pub const E1000_TIDV: ::kernel::sys::raw::c_uint = 14368;
// pub const E1000_TADV: ::kernel::sys::raw::c_uint = 14380;
// pub const E1000_TSPMT: ::kernel::sys::raw::c_uint = 14384;
// pub const E1000_CRCERRS: ::kernel::sys::raw::c_uint = 16384;
// pub const E1000_ALGNERRC: ::kernel::sys::raw::c_uint = 16388;
// pub const E1000_SYMERRS: ::kernel::sys::raw::c_uint = 16392;
// pub const E1000_RXERRC: ::kernel::sys::raw::c_uint = 16396;
// pub const E1000_MPC: ::kernel::sys::raw::c_uint = 16400;
// pub const E1000_SCC: ::kernel::sys::raw::c_uint = 16404;
// pub const E1000_ECOL: ::kernel::sys::raw::c_uint = 16408;
// pub const E1000_MCC: ::kernel::sys::raw::c_uint = 16412;
// pub const E1000_LATECOL: ::kernel::sys::raw::c_uint = 16416;
// pub const E1000_COLC: ::kernel::sys::raw::c_uint = 16424;
// pub const E1000_DC: ::kernel::sys::raw::c_uint = 16432;
// pub const E1000_TNCRS: ::kernel::sys::raw::c_uint = 16436;
// pub const E1000_SEC: ::kernel::sys::raw::c_uint = 16440;
// pub const E1000_CEXTERR: ::kernel::sys::raw::c_uint = 16444;
// pub const E1000_RLEC: ::kernel::sys::raw::c_uint = 16448;
// pub const E1000_XONRXC: ::kernel::sys::raw::c_uint = 16456;
// pub const E1000_XONTXC: ::kernel::sys::raw::c_uint = 16460;
// pub const E1000_XOFFRXC: ::kernel::sys::raw::c_uint = 16464;
// pub const E1000_XOFFTXC: ::kernel::sys::raw::c_uint = 16468;
// pub const E1000_FCRUC: ::kernel::sys::raw::c_uint = 16472;
// pub const E1000_PRC64: ::kernel::sys::raw::c_uint = 16476;
// pub const E1000_PRC127: ::kernel::sys::raw::c_uint = 16480;
// pub const E1000_PRC255: ::kernel::sys::raw::c_uint = 16484;
// pub const E1000_PRC511: ::kernel::sys::raw::c_uint = 16488;
// pub const E1000_PRC1023: ::kernel::sys::raw::c_uint = 16492;
// pub const E1000_PRC1522: ::kernel::sys::raw::c_uint = 16496;
// pub const E1000_GPRC: ::kernel::sys::raw::c_uint = 16500;
// pub const E1000_BPRC: ::kernel::sys::raw::c_uint = 16504;
// pub const E1000_MPRC: ::kernel::sys::raw::c_uint = 16508;
// pub const E1000_GPTC: ::kernel::sys::raw::c_uint = 16512;
// pub const E1000_GORCL: ::kernel::sys::raw::c_uint = 16520;
// pub const E1000_GORCH: ::kernel::sys::raw::c_uint = 16524;
// pub const E1000_GOTCL: ::kernel::sys::raw::c_uint = 16528;
// pub const E1000_GOTCH: ::kernel::sys::raw::c_uint = 16532;
// pub const E1000_RNBC: ::kernel::sys::raw::c_uint = 16544;
// pub const E1000_RUC: ::kernel::sys::raw::c_uint = 16548;
// pub const E1000_RFC: ::kernel::sys::raw::c_uint = 16552;
// pub const E1000_ROC: ::kernel::sys::raw::c_uint = 16556;
// pub const E1000_RJC: ::kernel::sys::raw::c_uint = 16560;
// pub const E1000_MGTPRC: ::kernel::sys::raw::c_uint = 16564;
// pub const E1000_MGTPDC: ::kernel::sys::raw::c_uint = 16568;
// pub const E1000_MGTPTC: ::kernel::sys::raw::c_uint = 16572;
// pub const E1000_TORL: ::kernel::sys::raw::c_uint = 16576;
// pub const E1000_TORH: ::kernel::sys::raw::c_uint = 16580;
// pub const E1000_TOTL: ::kernel::sys::raw::c_uint = 16584;
// pub const E1000_TOTH: ::kernel::sys::raw::c_uint = 16588;
// pub const E1000_TPR: ::kernel::sys::raw::c_uint = 16592;
// pub const E1000_TPT: ::kernel::sys::raw::c_uint = 16596;
// pub const E1000_PTC64: ::kernel::sys::raw::c_uint = 16600;
// pub const E1000_PTC127: ::kernel::sys::raw::c_uint = 16604;
// pub const E1000_PTC255: ::kernel::sys::raw::c_uint = 16608;
// pub const E1000_PTC511: ::kernel::sys::raw::c_uint = 16612;
// pub const E1000_PTC1023: ::kernel::sys::raw::c_uint = 16616;
// pub const E1000_PTC1522: ::kernel::sys::raw::c_uint = 16620;
// pub const E1000_MPTC: ::kernel::sys::raw::c_uint = 16624;
// pub const E1000_BPTC: ::kernel::sys::raw::c_uint = 16628;
// pub const E1000_TSCTC: ::kernel::sys::raw::c_uint = 16632;
// pub const E1000_TSCTFC: ::kernel::sys::raw::c_uint = 16636;
// pub const E1000_IAC: ::kernel::sys::raw::c_uint = 16640;
// pub const E1000_ICRXPTC: ::kernel::sys::raw::c_uint = 16644;
// pub const E1000_ICRXATC: ::kernel::sys::raw::c_uint = 16648;
// pub const E1000_ICTXPTC: ::kernel::sys::raw::c_uint = 16652;
// pub const E1000_ICTXATC: ::kernel::sys::raw::c_uint = 16656;
// pub const E1000_ICTXQEC: ::kernel::sys::raw::c_uint = 16664;
// pub const E1000_ICTXQMTC: ::kernel::sys::raw::c_uint = 16668;
// pub const E1000_ICRXDMTC: ::kernel::sys::raw::c_uint = 16672;
// pub const E1000_ICRXOC: ::kernel::sys::raw::c_uint = 16676;
// pub const E1000_CRC_OFFSET: ::kernel::sys::raw::c_uint = 24400;
// pub const E1000_VFGPRC: ::kernel::sys::raw::c_uint = 3856;
// pub const E1000_VFGORC: ::kernel::sys::raw::c_uint = 3864;
// pub const E1000_VFMPRC: ::kernel::sys::raw::c_uint = 3900;
// pub const E1000_VFGPTC: ::kernel::sys::raw::c_uint = 3860;
// pub const E1000_VFGOTC: ::kernel::sys::raw::c_uint = 3892;
// pub const E1000_VFGOTLBC: ::kernel::sys::raw::c_uint = 3920;
// pub const E1000_VFGPTLBC: ::kernel::sys::raw::c_uint = 3908;
// pub const E1000_VFGORLBC: ::kernel::sys::raw::c_uint = 3912;
// pub const E1000_VFGPRLBC: ::kernel::sys::raw::c_uint = 3904;
// pub const E1000_LSECTXUT: ::kernel::sys::raw::c_uint = 17152;
// pub const E1000_LSECTXPKTE: ::kernel::sys::raw::c_uint = 17156;
// pub const E1000_LSECTXPKTP: ::kernel::sys::raw::c_uint = 17160;
// pub const E1000_LSECTXOCTE: ::kernel::sys::raw::c_uint = 17164;
// pub const E1000_LSECTXOCTP: ::kernel::sys::raw::c_uint = 17168;
// pub const E1000_LSECRXUT: ::kernel::sys::raw::c_uint = 17172;
// pub const E1000_LSECRXOCTD: ::kernel::sys::raw::c_uint = 17180;
// pub const E1000_LSECRXOCTV: ::kernel::sys::raw::c_uint = 17184;
// pub const E1000_LSECRXBAD: ::kernel::sys::raw::c_uint = 17188;
// pub const E1000_LSECRXNOSCI: ::kernel::sys::raw::c_uint = 17192;
// pub const E1000_LSECRXUNSCI: ::kernel::sys::raw::c_uint = 17196;
// pub const E1000_LSECRXUNCH: ::kernel::sys::raw::c_uint = 17200;
// pub const E1000_LSECRXDELAY: ::kernel::sys::raw::c_uint = 17216;
// pub const E1000_LSECRXLATE: ::kernel::sys::raw::c_uint = 17232;
// pub const E1000_LSECRXUNSA: ::kernel::sys::raw::c_uint = 17344;
// pub const E1000_LSECRXNUSA: ::kernel::sys::raw::c_uint = 17360;
// pub const E1000_LSECTXCAP: ::kernel::sys::raw::c_uint = 45056;
// pub const E1000_LSECRXCAP: ::kernel::sys::raw::c_uint = 45824;
// pub const E1000_LSECTXCTRL: ::kernel::sys::raw::c_uint = 45060;
// pub const E1000_LSECRXCTRL: ::kernel::sys::raw::c_uint = 45828;
// pub const E1000_LSECTXSCL: ::kernel::sys::raw::c_uint = 45064;
// pub const E1000_LSECTXSCH: ::kernel::sys::raw::c_uint = 45068;
// pub const E1000_LSECTXSA: ::kernel::sys::raw::c_uint = 45072;
// pub const E1000_LSECTXPN0: ::kernel::sys::raw::c_uint = 45080;
// pub const E1000_LSECTXPN1: ::kernel::sys::raw::c_uint = 45084;
// pub const E1000_LSECRXSCL: ::kernel::sys::raw::c_uint = 46032;
// pub const E1000_LSECRXSCH: ::kernel::sys::raw::c_uint = 46048;
// pub const E1000_SSVPC: ::kernel::sys::raw::c_uint = 16800;
// pub const E1000_IPSCTRL: ::kernel::sys::raw::c_uint = 46128;
// pub const E1000_IPSRXCMD: ::kernel::sys::raw::c_uint = 46088;
// pub const E1000_IPSRXIDX: ::kernel::sys::raw::c_uint = 46080;
// pub const E1000_IPSRXSALT: ::kernel::sys::raw::c_uint = 46084;
// pub const E1000_IPSRXSPI: ::kernel::sys::raw::c_uint = 46092;
// pub const E1000_IPSTXSALT: ::kernel::sys::raw::c_uint = 46164;
// pub const E1000_IPSTXIDX: ::kernel::sys::raw::c_uint = 46160;
// pub const E1000_PCS_CFG0: ::kernel::sys::raw::c_uint = 16896;
// pub const E1000_PCS_LCTL: ::kernel::sys::raw::c_uint = 16904;
// pub const E1000_PCS_LSTAT: ::kernel::sys::raw::c_uint = 16908;
// pub const E1000_CBTMPC: ::kernel::sys::raw::c_uint = 16428;
// pub const E1000_HTDPMC: ::kernel::sys::raw::c_uint = 16444;
// pub const E1000_CBRDPC: ::kernel::sys::raw::c_uint = 16452;
// pub const E1000_CBRMPC: ::kernel::sys::raw::c_uint = 16636;
// pub const E1000_RPTHC: ::kernel::sys::raw::c_uint = 16644;
// pub const E1000_HGPTC: ::kernel::sys::raw::c_uint = 16664;
// pub const E1000_HTCBDPC: ::kernel::sys::raw::c_uint = 16676;
// pub const E1000_HGORCL: ::kernel::sys::raw::c_uint = 16680;
// pub const E1000_HGORCH: ::kernel::sys::raw::c_uint = 16684;
// pub const E1000_HGOTCL: ::kernel::sys::raw::c_uint = 16688;
// pub const E1000_HGOTCH: ::kernel::sys::raw::c_uint = 16692;
// pub const E1000_LENERRS: ::kernel::sys::raw::c_uint = 16696;
// pub const E1000_SCVPC: ::kernel::sys::raw::c_uint = 16936;
// pub const E1000_HRMPC: ::kernel::sys::raw::c_uint = 40984;
// pub const E1000_PCS_ANADV: ::kernel::sys::raw::c_uint = 16920;
// pub const E1000_PCS_LPAB: ::kernel::sys::raw::c_uint = 16924;
// pub const E1000_PCS_NPTX: ::kernel::sys::raw::c_uint = 16928;
// pub const E1000_PCS_LPABNP: ::kernel::sys::raw::c_uint = 16932;
// pub const E1000_RXCSUM: ::kernel::sys::raw::c_uint = 20480;
// pub const E1000_RLPML: ::kernel::sys::raw::c_uint = 20484;
// pub const E1000_RFCTL: ::kernel::sys::raw::c_uint = 20488;
// pub const E1000_MTA: ::kernel::sys::raw::c_uint = 20992;
// pub const E1000_RA: ::kernel::sys::raw::c_uint = 21504;
// pub const E1000_RA2: ::kernel::sys::raw::c_uint = 21728;
// pub const E1000_VFTA: ::kernel::sys::raw::c_uint = 22016;
// pub const E1000_VT_CTL: ::kernel::sys::raw::c_uint = 22556;
// pub const E1000_CIAA: ::kernel::sys::raw::c_uint = 23432;
// pub const E1000_CIAD: ::kernel::sys::raw::c_uint = 23436;
// pub const E1000_VFQA0: ::kernel::sys::raw::c_uint = 45056;
// pub const E1000_VFQA1: ::kernel::sys::raw::c_uint = 45568;
// pub const E1000_WUC: ::kernel::sys::raw::c_uint = 22528;
// pub const E1000_WUFC: ::kernel::sys::raw::c_uint = 22536;
// pub const E1000_WUS: ::kernel::sys::raw::c_uint = 22544;
// pub const E1000_MANC: ::kernel::sys::raw::c_uint = 22560;
// pub const E1000_IPAV: ::kernel::sys::raw::c_uint = 22584;
// pub const E1000_IP4AT: ::kernel::sys::raw::c_uint = 22592;
// pub const E1000_IP6AT: ::kernel::sys::raw::c_uint = 22656;
// pub const E1000_WUPL: ::kernel::sys::raw::c_uint = 22784;
// pub const E1000_WUPM: ::kernel::sys::raw::c_uint = 23040;
// pub const E1000_PBACL: ::kernel::sys::raw::c_uint = 23400;
// pub const E1000_FFLT: ::kernel::sys::raw::c_uint = 24320;
// pub const E1000_HOST_IF: ::kernel::sys::raw::c_uint = 34816;
// pub const E1000_HIBBA: ::kernel::sys::raw::c_uint = 36672;
// pub const E1000_KMRNCTRLSTA: ::kernel::sys::raw::c_uint = 52;
// pub const E1000_MANC2H: ::kernel::sys::raw::c_uint = 22624;
// pub const E1000_SW_FW_SYNC: ::kernel::sys::raw::c_uint = 23388;
// pub const E1000_CCMCTL: ::kernel::sys::raw::c_uint = 23368;
// pub const E1000_GIOCTL: ::kernel::sys::raw::c_uint = 23364;
// pub const E1000_SCCTL: ::kernel::sys::raw::c_uint = 23372;
// pub const E1000_GCR: ::kernel::sys::raw::c_uint = 23296;
// pub const E1000_GCR2: ::kernel::sys::raw::c_uint = 23396;
// pub const E1000_GSCL_1: ::kernel::sys::raw::c_uint = 23312;
// pub const E1000_GSCL_2: ::kernel::sys::raw::c_uint = 23316;
// pub const E1000_GSCL_3: ::kernel::sys::raw::c_uint = 23320;
// pub const E1000_GSCL_4: ::kernel::sys::raw::c_uint = 23324;
// pub const E1000_FACTPS: ::kernel::sys::raw::c_uint = 23344;
// pub const E1000_SWSM: ::kernel::sys::raw::c_uint = 23376;
// pub const E1000_FWSM: ::kernel::sys::raw::c_uint = 23380;
// pub const E1000_SWSM2: ::kernel::sys::raw::c_uint = 23384;
// pub const E1000_DCA_ID: ::kernel::sys::raw::c_uint = 23408;
// pub const E1000_DCA_CTRL: ::kernel::sys::raw::c_uint = 23412;
// pub const E1000_UFUSE: ::kernel::sys::raw::c_uint = 23416;
// pub const E1000_FFLT_DBG: ::kernel::sys::raw::c_uint = 24324;
// pub const E1000_HICR: ::kernel::sys::raw::c_uint = 36608;
// pub const E1000_FWSTS: ::kernel::sys::raw::c_uint = 36620;
// pub const E1000_CPUVEC: ::kernel::sys::raw::c_uint = 11280;
// pub const E1000_MRQC: ::kernel::sys::raw::c_uint = 22552;
// pub const E1000_IMIRVP: ::kernel::sys::raw::c_uint = 23232;
// pub const E1000_RSSIM: ::kernel::sys::raw::c_uint = 22628;
// pub const E1000_RSSIR: ::kernel::sys::raw::c_uint = 22632;
// pub const E1000_SWPBS: ::kernel::sys::raw::c_uint = 12292;
// pub const E1000_MBVFICR: ::kernel::sys::raw::c_uint = 3200;
// pub const E1000_MBVFIMR: ::kernel::sys::raw::c_uint = 3204;
// pub const E1000_VFLRE: ::kernel::sys::raw::c_uint = 3208;
// pub const E1000_VFRE: ::kernel::sys::raw::c_uint = 3212;
// pub const E1000_VFTE: ::kernel::sys::raw::c_uint = 3216;
// pub const E1000_QDE: ::kernel::sys::raw::c_uint = 9224;
// pub const E1000_DTXSWC: ::kernel::sys::raw::c_uint = 13568;
// pub const E1000_WVBR: ::kernel::sys::raw::c_uint = 13652;
// pub const E1000_RPLOLR: ::kernel::sys::raw::c_uint = 23280;
// pub const E1000_UTA: ::kernel::sys::raw::c_uint = 40960;
// pub const E1000_IOVCTL: ::kernel::sys::raw::c_uint = 23484;
// pub const E1000_VMRVLAN: ::kernel::sys::raw::c_uint = 23952;
// pub const E1000_VMRVM: ::kernel::sys::raw::c_uint = 23968;
// pub const E1000_MDFB: ::kernel::sys::raw::c_uint = 13656;
// pub const E1000_LVMMC: ::kernel::sys::raw::c_uint = 13640;
// pub const E1000_TXSWC: ::kernel::sys::raw::c_uint = 23244;
// pub const E1000_SCCRL: ::kernel::sys::raw::c_uint = 23984;
// pub const E1000_BSCTRH: ::kernel::sys::raw::c_uint = 23992;
// pub const E1000_MSCTRH: ::kernel::sys::raw::c_uint = 23996;
// pub const E1000_TSYNCRXCTL: ::kernel::sys::raw::c_uint = 46624;
// pub const E1000_TSYNCTXCTL: ::kernel::sys::raw::c_uint = 46612;
// pub const E1000_TSYNCRXCFG: ::kernel::sys::raw::c_uint = 24400;
// pub const E1000_RXSTMPL: ::kernel::sys::raw::c_uint = 46628;
// pub const E1000_RXSTMPH: ::kernel::sys::raw::c_uint = 46632;
// pub const E1000_RXSATRL: ::kernel::sys::raw::c_uint = 46636;
// pub const E1000_RXSATRH: ::kernel::sys::raw::c_uint = 46640;
// pub const E1000_TXSTMPL: ::kernel::sys::raw::c_uint = 46616;
// pub const E1000_TXSTMPH: ::kernel::sys::raw::c_uint = 46620;
// pub const E1000_SYSTIML: ::kernel::sys::raw::c_uint = 46592;
// pub const E1000_SYSTIMH: ::kernel::sys::raw::c_uint = 46596;
// pub const E1000_TIMINCA: ::kernel::sys::raw::c_uint = 46600;
// pub const E1000_TIMADJL: ::kernel::sys::raw::c_uint = 46604;
// pub const E1000_TIMADJH: ::kernel::sys::raw::c_uint = 46608;
// pub const E1000_TSAUXC: ::kernel::sys::raw::c_uint = 46656;
// pub const E1000_SYSSTMPL: ::kernel::sys::raw::c_uint = 46664;
// pub const E1000_SYSSTMPH: ::kernel::sys::raw::c_uint = 46668;
// pub const E1000_PLTSTMPL: ::kernel::sys::raw::c_uint = 46656;
// pub const E1000_PLTSTMPH: ::kernel::sys::raw::c_uint = 46660;
// pub const E1000_SYSTIMR: ::kernel::sys::raw::c_uint = 46840;
// pub const E1000_TSICR: ::kernel::sys::raw::c_uint = 46700;
// pub const E1000_TSIM: ::kernel::sys::raw::c_uint = 46708;
// pub const E1000_RXMTRL: ::kernel::sys::raw::c_uint = 46644;
// pub const E1000_RXUDP: ::kernel::sys::raw::c_uint = 46648;
// pub const E1000_RTTDCS: ::kernel::sys::raw::c_uint = 13824;
// pub const E1000_RTTPCS: ::kernel::sys::raw::c_uint = 13428;
// pub const E1000_RTRPCS: ::kernel::sys::raw::c_uint = 9332;
// pub const E1000_RTRUP2TC: ::kernel::sys::raw::c_uint = 23236;
// pub const E1000_RTTUP2TC: ::kernel::sys::raw::c_uint = 1048;
// pub const E1000_RTTDQSEL: ::kernel::sys::raw::c_uint = 13828;
// pub const E1000_RTTDVMRC: ::kernel::sys::raw::c_uint = 13832;
// pub const E1000_RTTDVMRS: ::kernel::sys::raw::c_uint = 13836;
// pub const E1000_RTTBCNRC: ::kernel::sys::raw::c_uint = 14000;
// pub const E1000_RTTBCNRS: ::kernel::sys::raw::c_uint = 14004;
// pub const E1000_RTTBCNCR: ::kernel::sys::raw::c_uint = 45568;
// pub const E1000_RTTBCNTG: ::kernel::sys::raw::c_uint = 13732;
// pub const E1000_RTTBCNCP: ::kernel::sys::raw::c_uint = 45576;
// pub const E1000_RTRBCNCR: ::kernel::sys::raw::c_uint = 45580;
// pub const E1000_RTTBCNRD: ::kernel::sys::raw::c_uint = 14008;
// pub const E1000_PFCTOP: ::kernel::sys::raw::c_uint = 4224;
// pub const E1000_RTTBCNIDX: ::kernel::sys::raw::c_uint = 45572;
// pub const E1000_RTTBCNACH: ::kernel::sys::raw::c_uint = 45588;
// pub const E1000_RTTBCNACL: ::kernel::sys::raw::c_uint = 45584;
// pub const E1000_DMACR: ::kernel::sys::raw::c_uint = 9480;
// pub const E1000_DMCTXTH: ::kernel::sys::raw::c_uint = 13648;
// pub const E1000_DMCTLX: ::kernel::sys::raw::c_uint = 9492;
// pub const E1000_DMCRTRH: ::kernel::sys::raw::c_uint = 24016;
// pub const E1000_DMCCNT: ::kernel::sys::raw::c_uint = 24020;
// pub const E1000_FCRTC: ::kernel::sys::raw::c_uint = 8560;
// pub const E1000_PCIEMISC: ::kernel::sys::raw::c_uint = 23480;
// pub const E1000_PCIEERRSTS: ::kernel::sys::raw::c_uint = 23464;
// pub const E1000_PROXYS: ::kernel::sys::raw::c_uint = 24420;
// pub const E1000_PROXYFC: ::kernel::sys::raw::c_uint = 24416;
// pub const E1000_THMJT: ::kernel::sys::raw::c_uint = 33024;
// pub const E1000_THLOWTC: ::kernel::sys::raw::c_uint = 33028;
// pub const E1000_THMIDTC: ::kernel::sys::raw::c_uint = 33032;
// pub const E1000_THHIGHTC: ::kernel::sys::raw::c_uint = 33036;
// pub const E1000_THSTAT: ::kernel::sys::raw::c_uint = 33040;
// pub const E1000_IPCNFG: ::kernel::sys::raw::c_uint = 3640;
// pub const E1000_LTRC: ::kernel::sys::raw::c_uint = 416;
// pub const E1000_EEER: ::kernel::sys::raw::c_uint = 3632;
// pub const E1000_EEE_SU: ::kernel::sys::raw::c_uint = 3636;
// pub const E1000_TLPIC: ::kernel::sys::raw::c_uint = 16712;
// pub const E1000_RLPIC: ::kernel::sys::raw::c_uint = 16716;
// pub const E1000_B2OSPC: ::kernel::sys::raw::c_uint = 36832;
// pub const E1000_B2OGPRC: ::kernel::sys::raw::c_uint = 16728;
// pub const E1000_O2BGPTC: ::kernel::sys::raw::c_uint = 36836;
// pub const E1000_O2BSPC: ::kernel::sys::raw::c_uint = 16732;
// pub const E1000_DOBFFCTL: ::kernel::sys::raw::c_uint = 16164;
