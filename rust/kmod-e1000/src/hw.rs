use kernel;
use kernel::fmt;
use kernel::sys::raw::*;
use kernel::prelude::v1::*;
use kernel::fmt::Debug;

use adapter::*;
use e1000_osdep::*;

use e1000_mac;
use e1000_mbx;
use e1000_phy;
use e1000_nvm;

use sys::e1000::*;

#[derive(Debug)]
pub struct Hardware {
    pub phy: PhyInfo,
    pub mac: MacInfo,
    pub nvm: NvmInfo,
    pub mbx: MbxInfo,
    pub bus: BusInfo,
    pub fc: FcInfo,

    pub vendor_id: u16,
    pub device_id: u16,
    pub subsystem_vendor_id: u16,
    pub subsystem_device_id: u16,
    pub revision_id: u8,
    pub memory: MappedMemory,
    pub io_base: u64,

    pub flash_address: *mut u8,
    pub hw_addr: *mut u8,

    pub dev_spec: DevSpec,
}
impl Hardware {}
// impl fmt::Debug for Hardware {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "Hardware {{ vendor_id: 0x{:4x}, device_id: 0x{:4x} }}",
//             self.vendor_id,
//             self.device_id
//         )
//     }
// }
impl Drop for Hardware {
    fn drop(&mut self) {
        e1000_println!("<<<<<<<<<<<<<<<<<<<< DROP >>>>>>>>>>>>>>>>>>>>");
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum FcMode {
    None = 0,
    RxPause = 1,
    TxPause = 2,
    Full = 3,
    Default = 255,
}
use kernel::ops::BitAnd;
impl BitAnd for FcMode {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        match (self as u32) & (rhs as u32) {
            x if x > 0 => rhs,
            _ => FcMode::None,
        }
    }
}

pub union DevSpec {
    pub _82541: DevSpec_82541,
    // pub _82542: e1000_dev_spec_82542,
    // pub _82543: e1000_dev_spec_82543,
    // pub _82571: e1000_dev_spec_82571,
    // pub _80003es2lan: e1000_dev_spec_80003es2lan,
    pub ich8lan: DevSpec_ich8lan,
    // pub _82575: e1000_dev_spec_82575,
    // pub vf: e1000_dev_spec_vf,
    _bindgen_union_align: [u64; 1035usize],
}
impl fmt::Debug for DevSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DevSpec {{ union }}")
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct DevSpec_82541 {
    // pub dsp_config: e1000_dsp_config,
    // pub ffe_config: e1000_ffe_config,
    // pub spd_default: u16,
    pub phy_init_script: bool,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct ShadowRam {
    pub value: u16,
    pub modified: bool,
}

// #[derive(Debug)]
// #[allow(non_camel_case_types)]
pub struct DevSpec_ich8lan {
    pub kmrn_lock_loss_workaround_enabled: bool,
    pub shadow_ram: [ShadowRam; 2048],
    pub nvm_k1_enabled: bool,
    pub disable_k1_off: bool,
    pub eee_disable: bool,
    pub eee_lp_ability: u16,
    pub ulp_state: UlpState,
    pub ulp_capability_disabled: bool,
    pub during_suspend_flow: bool,
    pub during_dpg_exit: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum UlpState {
    Unknown = 0,
    Off = 1,
    On = 2,
}

#[derive(Debug)]
pub struct MappedMemory {
    ptr: *mut u8,
    len: usize,
}
impl MappedMemory {
    pub fn new(ptr: *mut u8, len: usize) -> MappedMemory {
        e1000_println!();
        MappedMemory { ptr: ptr, len: len }
    }
}
impl kernel::ops::Deref for MappedMemory {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe { kernel::slice::from_raw_parts(self.ptr, self.len) }
    }
}
impl kernel::ops::DerefMut for MappedMemory {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { kernel::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}
impl Drop for MappedMemory {
    fn drop(&mut self) {
        e1000_println!("<<<<<<<<<<<<<<<<<<<< DROP >>>>>>>>>>>>>>>>>>>>");
        e1000_println!("ptr: {:?}", self.ptr);
        e1000_println!("len: {:?}", self.len);
    }
}
// impl fmt::Debug for MappedMemory {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "MappedMemory)
//     }
// }

#[derive(Debug)]
pub struct FcInfo {
    pub high_water: u32,
    pub low_water: u32,
    pub pause_time: u16,
    pub refresh_time: u16,
    pub send_xon: bool,
    pub strict_ieee: bool,
    pub current_mode: FcMode,
    pub requested_mode: FcMode,
}

#[derive(Debug)]
pub struct BusInfo {
    pub bustype: BusType,
    pub speed: BusSpeed,
    pub width: BusWidth,
    pub func: u16,
    pub pci_cmd_word: u16,
}

#[derive(Debug)]
pub struct PhyInfo {
    pub ops: PhyOps,
    pub media_type: MediaType,
    pub phy_type: PhyType,
    pub local_rx: GbRxStatus,
    pub remote_rx: GbRxStatus,
    pub ms_type: MsType,
    pub original_ms_type: MsType,
    pub cable_polarity: RevPolarity,
    pub smart_speed: SmartSpeed,
    pub addr: u32,
    pub id: u32,
    pub reset_delay_us: u32,
    pub revision: u32,
    pub autoneg_advertised: u16,
    pub autoneg_mask: u16,
    pub cable_length: u16,
    pub max_cable_length: u16,
    pub min_cable_length: u16,
    pub mdix: u8,
    pub disable_polarity_correction: bool,
    pub is_mdix: bool,
    pub polarity_correction: bool,
    pub speed_downgraded: bool,
    pub autoneg_wait_to_complete: bool,
}
impl PhyInfo {}
pub struct PhyOps {
    pub init_params: Option<AdFn>,
    pub acquire: Option<AdFn>,
    pub cfg_on_link_up: Option<AdFn>,
    pub check_polarity: Option<AdFn>,
    pub check_reset_block: Option<fn(hw: &mut Adapter) -> Result<bool, String>>,
    pub commit: Option<AdFn>,
    pub force_speed_duplex: Option<AdFn>,
    pub get_cfg_done: Option<fn(hw: &mut Adapter) -> AdResult>,
    pub get_cable_length: Option<AdFn>,
    pub get_info: Option<AdFn>,
    pub set_page: Option<fn(arg1: &mut Adapter, arg2: u16) -> AdResult>,
    pub read_reg: Option<fn(arg1: &mut Adapter, arg2: u32, arg3: &mut u16) -> AdResult>,
    pub read_reg_locked: Option<fn(arg1: &mut Adapter, arg2: u32, arg3: &mut u16) -> AdResult>,
    pub read_reg_page: Option<fn(arg1: &mut Adapter, arg2: u32, arg3: &mut u16) -> AdResult>,
    pub release: Option<AdFn>,
    pub reset: Option<AdFn>,
    pub set_d0_lplu_state: Option<fn(arg1: &mut Adapter, arg2: bool) -> AdResult>,
    pub set_d3_lplu_state: Option<fn(arg1: &mut Adapter, arg2: bool) -> AdResult>,
    pub write_reg: Option<fn(arg1: &mut Adapter, arg2: u32, arg3: u16) -> AdResult>,
    pub write_reg_locked: Option<fn(arg1: &mut Adapter, arg2: u32, arg3: u16) -> AdResult>,
    pub write_reg_page: Option<fn(arg1: &mut Adapter, arg2: u32, arg3: u16) -> AdResult>,
    pub power_up: Option<fn(arg1: &mut Adapter)>,
    pub power_down: Option<fn(arg1: &mut Adapter)>,
    pub read_i2c_byte:
        Option<fn(arg1: &mut Adapter, arg2: u8, arg3: u8, arg4: &mut u8) -> AdResult>,
    pub write_i2c_byte: Option<fn(arg1: &mut Adapter, arg2: u8, arg3: u8, arg4: u8) -> AdResult>,
}
impl PhyOps {
    pub fn init_generic(&mut self) {
        e1000_println!();
        // generic are all null ops
    }
}
impl fmt::Debug for PhyOps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PhyOps {{ ... }}")
    }
}

#[derive(Debug)]
pub struct NvmInfo {
    pub ops: NvmOps,
    pub nvm_type: NvmType,
    pub nvmoverride: NvmOverride,
    pub flash_bank_size: u32,
    pub flash_base_addr: u32,
    pub word_size: u16,
    pub delay_usec: u16,
    pub address_bits: u16,
    pub opcode_bits: u16,
    pub page_size: u16,
}
pub struct NvmOps {
    pub init_params: Option<AdFn>,
    pub acquire: Option<AdFn>,
    pub read:
        Option<fn(adapter: &mut Adapter, offset: u16, count: u16, data: &mut [u16]) -> AdResult>,
    pub release: Option<fn(arg1: &mut Adapter)>,
    pub reload: Option<fn(arg1: &mut Adapter)>,
    pub update: Option<AdFn>,
    pub valid_led_default: Option<fn(arg1: &mut Adapter, arg2: &mut [u16]) -> AdResult>,
    pub validate: Option<AdFn>,
    pub write: Option<fn(adapter: &mut Adapter, offset: u16, count: u16, data: &[u16]) -> AdResult>,
}
impl NvmOps {
    pub fn init_generic(&mut self) {
        e1000_println!();
        /*
	nvm->ops.reload = e1000_reload_nvm_generic;
         */
        self.reload = Some(e1000_nvm::reload_nvm_generic);
    }
}
impl fmt::Debug for NvmOps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NvmOps {{ ... }}")
    }
}

#[derive(Debug)]
pub struct MbxInfo {
    pub ops: MbxOps,
    pub stats: MbxStats,
    pub timeout: u32,
    pub usec_delay: u32,
    pub size: u16,
}
pub struct MbxOps {
    pub init_params: Option<AdFn>,
    pub read: Option<fn(arg1: &mut Adapter, arg2: &mut u32, arg3: u16, arg4: u16) -> AdResult>,
    pub write: Option<fn(arg1: &mut Adapter, arg2: &mut u32, arg3: u16, arg4: u16) -> AdResult>,
    pub read_posted:
        Option<fn(arg1: &mut Adapter, arg2: &mut u32, arg3: u16, arg4: u16) -> AdResult>,
    pub write_posted:
        Option<fn(arg1: &mut Adapter, arg2: &mut u32, arg3: u16, arg4: u16) -> AdResult>,
    pub check_for_msg: Option<fn(arg1: &mut Adapter, arg2: u16) -> AdResult>,
    pub check_for_ack: Option<fn(arg1: &mut Adapter, arg2: u16) -> AdResult>,
    pub check_for_rst: Option<fn(arg1: &mut Adapter, arg2: u16) -> AdResult>,
}
impl MbxOps {
    pub fn init_generic(&mut self) {
        e1000_println!();
        /*
	mbx->ops.read_posted = e1000_read_posted_mbx;
	mbx->ops.write_posted = e1000_write_posted_mbx;
         */
        self.read_posted = Some(e1000_mbx::read_posted);
        self.write_posted = Some(e1000_mbx::write_posted);
    }
}
impl fmt::Debug for MbxOps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MbxOps {{ ... }}")
    }
}

pub struct MacInfo {
    pub ops: MacOps,
    pub mac_type: MacType,
    pub addr: [u8; 6usize],
    pub perm_addr: [u8; 6usize],
    pub collision_delta: u32,
    pub ledctl_default: u32,
    pub ledctl_mode1: u32,
    pub ledctl_mode2: u32,
    pub mc_filter_type: u32,
    pub tx_packet_delta: u32,
    pub txcw: u32,
    pub current_ifs_val: u16,
    pub ifs_max_val: u16,
    pub ifs_min_val: u16,
    pub ifs_ratio: u16,
    pub ifs_step_size: u16,
    pub mta_reg_count: u16,
    pub uta_reg_count: u16,
    pub mta_shadow: [u32; 128usize],
    pub rar_entry_count: u16,
    pub forced_speed_duplex: u8,
    pub adaptive_ifs: bool,
    pub has_fwsm: bool,
    pub arc_subsystem_valid: bool,
    pub asf_firmware_present: bool,
    pub autoneg: bool,
    pub autoneg_failed: bool,
    pub get_link_status: bool,
    pub in_ifs_mode: bool,
    pub report_tx_early: bool,
    pub serdes_link_state: SerdesLinkState,
    pub serdes_has_link: bool,
    pub tx_pkt_filtering: bool,
    pub max_frame_size: u32,
}
impl fmt::Debug for MacInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MacInfo {{ mactype: {:?}, perm_addr: {:?} }}",
            self.mac_type, self.perm_addr
        )
    }
}
pub struct MacOps {
    pub init_params: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub id_led_init: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub blink_led: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub check_mng_mode: Option<fn(arg1: &mut Adapter) -> bool>,
    pub check_for_link: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub cleanup_led: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub clear_hw_cntrs: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub clear_vfta: Option<fn(arg1: &mut Adapter)>,
    pub get_bus_info: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub set_lan_id: Option<fn(arg1: &mut Adapter)>,
    pub get_link_up_info:
        Option<fn(arg1: &mut Adapter, arg2: &mut u16, arg3: &mut u16) -> AdResult>,
    pub led_on: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub led_off: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub update_mc_addr_list: Option<fn(adapter: &mut Adapter, mc_addr_count: u32) -> AdResult>,
    pub reset_hw: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub init_hw: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub shutdown_serdes: Option<fn(arg1: &mut Adapter)>,
    pub power_up_serdes: Option<fn(arg1: &mut Adapter)>,
    pub setup_link: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub setup_physical_interface: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub setup_led: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub write_vfta: Option<fn(arg1: &mut Adapter, arg2: u32, arg3: u32)>,
    pub config_collision_dist: Option<fn(arg1: &mut Adapter)>,
    pub rar_set: Option<fn(adapter: &mut Adapter, addr: &[u8], index: usize) -> AdResult>,
    pub read_mac_addr: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub validate_mdi_setting: Option<fn(arg1: &mut Adapter) -> AdResult>,
    pub set_obff_timer: Option<fn(arg1: &mut Adapter, arg2: u32) -> AdResult>,
    pub acquire_swfw_sync: Option<fn(arg1: &mut Adapter, arg2: u16) -> AdResult>,
    pub release_swfw_sync: Option<fn(arg1: &mut Adapter, arg2: u16)>,
}
impl MacOps {
    pub fn init_generic(&mut self) {
        e1000_println!();
        /* General Setup */
        // mac->ops.set_lan_id = e1000_set_lan_id_multi_port_pcie;
        self.set_lan_id = Some(e1000_mac::set_lan_id_multi_port_pcie);
        // mac->ops.read_mac_addr = e1000_read_mac_addr_generic;
        self.read_mac_addr = Some(e1000_nvm::read_mac_addr_generic);
        // mac->ops.config_collision_dist = e1000_config_collision_dist_generic;
        self.config_collision_dist = Some(e1000_mac::config_collision_dist_generic);

        /* LED */
        /* LINK */
        /* Management */
        /* VLAN, MC, etc. */
        // mac->ops.rar_set = e1000_rar_set_generic;
        self.rar_set = Some(e1000_mac::rar_set_generic);
        // mac->ops.validate_mdi_setting = e1000_validate_mdi_setting_generic;
        self.validate_mdi_setting = Some(e1000_mac::validate_mdi_setting_generic);
    }
}
impl fmt::Debug for MacOps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MacOps {{ ... }}")
    }
}
