use kernel;
use kernel::ptr::Unique;

use kernel::sys::raw::*;
use kernel::prelude::v1::*;

use kernel::sys::kernel_sys::caddr_t;
use kernel::sys::iflib_sys::bus_dma_segment;
use kernel::sys::iflib_sys::bus_dma_segment_t;
use kernel::sys::iflib_sys::if_rxd_frag;
use kernel::sys::iflib_sys::device_t;
use kernel::sys::iflib_sys::QIDX_INVALID;
use kernel::sys::bus_sys::bus_addr_t; // u64
use kernel::sys::bus_sys::bus_size_t; // u64
use kernel::sys::pci_sys::PCIR_COMMAND;
use kernel::sys::pci_sys::PCIR_REVID;
use kernel::sys::pci_sys::PCIR_SUBVEND_0;
use kernel::sys::pci_sys::PCIR_SUBDEV_0;
use kernel::sys::pci_sys::PCIR_CIS;

use kernel::sys::iflib_sys;
use kernel::sys::iflib_sys::ether_vlan_header;
use kernel::sys::iflib_sys::{IFCAP_TSO4, IFCAP_HWCSUM, IFCAP_LRO, IFCAP_RXCSUM, IFCAP_TXCSUM,
                             IFCAP_VLAN_HWCSUM, IFCAP_VLAN_HWFILTER, IFCAP_VLAN_HWTAGGING,
                             IFCAP_VLAN_HWTSO, IFCAP_VLAN_MTU, IFCAP_WOL, IFCAP_WOL_MAGIC,
                             IFCAP_WOL_MCAST, IFCAP_WOL_UCAST};
use kernel::sys::iflib_sys::ETHERTYPE_VLAN;
use kernel::sys::iflib_sys::LINK_STATE_DOWN;
use kernel::sys::iflib_sys::LINK_STATE_UP;
use kernel::sys::iflib_sys::LINK_STATE_UNKNOWN;
use kernel::sys::iflib_sys::ETHERMTU;
use kernel::sys::iflib_sys::IFF_PROMISC;
use kernel::sys::iflib_sys::IFF_ALLMULTI;
use kernel::sys::iflib_sys::IPI_TX_INTR;
use kernel::sys::iflib_sys::M_VLANTAG;
use kernel::sys::iflib_sys::CSUM_TSO;
use kernel::sys::iflib_sys::CSUM_TCP;
use kernel::sys::iflib_sys::CSUM_UDP;
use kernel::sys::iflib_sys::CSUM_IP;
use kernel::sys::iflib_sys::CSUM_PSEUDO_HDR;
use kernel::sys::iflib_sys::CSUM_DATA_VALID;
use kernel::sys::iflib_sys::CSUM_IP_VALID;
use kernel::sys::iflib_sys::CSUM_IP_CHECKED;
use kernel::sys::iflib_sys::FILTER_SCHEDULE_THREAD;
use kernel::sys::iflib_sys::FILTER_STRAY;
use kernel::sys::iflib_sys::FILTER_HANDLED;

use sys::iflib::iflib_intr_mode_t;

use sys::e1000::*;
use sys::e1000_consts::*;
use iflib::*;
use hw::*;
use consts::*;
use bridge::*;

use e1000_regs::*;
use e1000_osdep::*;
use e1000_manage;
use e1000_82540;
use e1000_82541;
use e1000_82542;
use e1000_82543;
use e1000_ich8lan;
use e1000_nvm;
use e1000_phy;
use e1000_mac;

use log;

pub type AdResult = Result<(), String>;
pub type AdFn = fn(&mut Adapter) -> AdResult;

// #define MAX_INTS_PER_SEC	8000
// #define DEFAULT_ITR		(1000000000/(MAX_INTS_PER_SEC * 256))

const MAX_INTS_PER_SEC: u32 = 8000;
const DEFAULT_ITR: u32 = 1000000000 / (MAX_INTS_PER_SEC * 256);

#[derive(Debug)]
pub struct Adapter {
    pub dev: PciDevice,
    pub ifnet: IfNet,
    pub iflib: IfLib,
    pub iflib_shared: IfLibShared,
    pub ifmedia: IfMedia,
    pub hw: Hardware,
    pub osdep: OsDep,

    pub memory: Option<Resource>,
    pub ioport: Option<Resource>,
    pub flash: Option<Resource>,
    pub io_rid: i32,
    pub tx_process_limit: u32,
    pub rx_process_limit: u32,

    /* Multicast array memory */
    pub mta: Box<[u8]>,

    /* RX, TX */
    pub tx_queues: Box<[TxQueue]>,
    pub rx_queues: Box<[RxQueue]>,

    /* Management and WOL features */
    pub wol: u32,
    pub has_manage: bool,
    pub has_amt: bool,

    /* Info about the interface */
    pub link_active: bool,
    pub link_speed: u16,
    pub link_duplex: u16,
    pub smartspeed: u32,
    pub dmac: u32,
    pub link_mask: i32,
    pub que_mask: u64,

    /* Misc stats maintained by the driver */
    pub dropped_pkts: u64,
    pub link_irq: u64,
    pub mbuf_defrag_failed: u64,
    pub no_tx_dma_setup: u64,
    pub no_tx_map_avail: u64,
    pub rx_overruns: u64,
    pub watchdog_events: u64,

    pub tx_int_delay: IntDelayInfo,
    pub tx_abs_int_delay: IntDelayInfo,

    pub rx_int_delay: IntDelayInfo,
    pub rx_abs_int_delay: IntDelayInfo,

    pub fc: FcMode,
    pub stats: HwStats,
    pub vf_ifp: u16,
    pub txd_cmd: u32,
    pub rx_mbuf_sz: u32,
    pub num_vlans: u32,
}

impl Adapter {
    pub fn read_register(&self, reg: u32) -> u32 {
        unsafe {
            rust_bus_space_read_4(
                self.osdep.mem_bus_space_tag,
                self.osdep.mem_bus_space_handle,
                reg as bus_size_t,
            )
        }
    }

    pub fn write_register(&self, reg: u32, value: u32) {
        unsafe {
            rust_bus_space_write_4(
                self.osdep.mem_bus_space_tag,
                self.osdep.mem_bus_space_handle,
                reg as bus_size_t,
                value,
            );
        }
    }

    pub fn write_flush(&self) {
        self.read_register(E1000_STATUS);
    }

    // #define E1000_READ_FLASH_REG16(hw, reg) \
    // bus_space_read_2(((struct e1000_osdep *)(hw)->back)->flash_bus_space_tag, \
    //                  ((struct e1000_osdep *)(hw)->back)->flash_bus_space_handle, reg)
    pub fn read_flash_register16(&self, reg: u32) -> u16 {
        unsafe {
            rust_bus_space_read_2(
                self.osdep.flash_bus_space_tag,
                self.osdep.flash_bus_space_handle,
                reg as bus_size_t,
            )
        }
    }

    // #define E1000_WRITE_FLASH_REG(hw, reg, value) \
    // bus_space_write_4(((struct e1000_osdep *)(hw)->back)->flash_bus_space_tag, \
    //                   ((struct e1000_osdep *)(hw)->back)->flash_bus_space_handle, reg, value)
    pub fn write_flash_register16(&self, reg: u32, value: u16) {
        unsafe {
            rust_bus_space_write_2(
                self.osdep.flash_bus_space_tag,
                self.osdep.flash_bus_space_handle,
                reg as bus_size_t,
                value,
            );
        }
    }

    // #define E1000_READ_FLASH_REG(hw, reg) \
    // bus_space_read_4(((struct e1000_osdep *)(hw)->back)->flash_bus_space_tag, \
    //                  ((struct e1000_osdep *)(hw)->back)->flash_bus_space_handle, reg)
    pub fn read_flash_register(&self, reg: u32) -> u32 {
        unsafe {
            rust_bus_space_read_4(
                self.osdep.flash_bus_space_tag,
                self.osdep.flash_bus_space_handle,
                reg as bus_size_t,
            )
        }
    }

    // #define E1000_WRITE_FLASH_REG(hw, reg, value) \
    // bus_space_write_4(((struct e1000_osdep *)(hw)->back)->flash_bus_space_tag, \
    //                   ((struct e1000_osdep *)(hw)->back)->flash_bus_space_handle, reg, value)
    pub fn write_flash_register(&self, reg: u32, value: u32) {
        unsafe {
            rust_bus_space_write_4(
                self.osdep.flash_bus_space_tag,
                self.osdep.flash_bus_space_handle,
                reg as bus_size_t,
                value,
            );
        }
    }

    pub fn clear_register_bit(&self, reg: u32, bit: u32) {
        let mut v: u32 = self.read_register(reg);
        v &= !bit;
        self.write_register(reg, v);
    }
    pub fn set_register_bit(&self, reg: u32, bit: u32) {
        let mut v: u32 = self.read_register(reg);
        v |= bit;
        self.write_register(reg, v);
    }
    pub fn is_copper(&self) -> bool {
        self.hw.phy.media_type == MediaType::Copper
    }
    pub fn is_mac(&self, mac_type: MacType) -> bool {
        self.hw.mac.mac_type == mac_type
    }
    pub fn is_macs(&self, mac_types: &[MacType]) -> bool {
        for t in mac_types {
            if t == &self.hw.mac.mac_type {
                return true;
            }
        }
        false
    }
    pub fn is_not_macs(&self, mac_types: &[MacType]) -> bool {
        for t in mac_types {
            if t == &self.hw.mac.mac_type {
                return false;
            }
        }
        true
    }
    pub fn rx_num_queues(&self) -> usize {
        self.iflib_shared.isc_nrxqsets as usize
    }
    pub fn tx_num_queues(&self) -> usize {
        self.iflib_shared.isc_ntxqsets as usize
    }
    pub fn identify_hardware(&mut self) -> AdResult {
        e1000_println!();

        /* Make sure our PCI config space has the necessary stuff set */
        self.hw.bus.pci_cmd_word = self.dev.pci_read_config(PCIR_COMMAND, 2) as u16;

        /* Save off the information about this board */
        self.hw.vendor_id = self.dev.pci_get_vendor() as u16;
        self.hw.device_id = self.dev.pci_get_device() as u16;
        self.hw.revision_id = self.dev.pci_read_config(PCIR_REVID, 1) as u8;
        self.hw.subsystem_vendor_id = self.dev.pci_read_config(PCIR_SUBVEND_0, 2) as u16;
        self.hw.subsystem_device_id = self.dev.pci_read_config(PCIR_SUBDEV_0, 2) as u16;

        /* Do Shared Code Init and Setup */
        try!(self.set_mac_type());
        Ok(())
    }

    pub fn set_mac_type(&mut self) -> AdResult {
        e1000_println!();

        // case E1000_DEV_ID_82545EM_COPPER:		/* bhyve */
        // case E1000_DEV_ID_82545EM_FIBER:
        // 	mac->type = e1000_82545;
        //      break;
        // case E1000_DEV_ID_PCH_LPT_I217_LM:
        // case E1000_DEV_ID_PCH_LPT_I217_V:
        // case E1000_DEV_ID_PCH_LPTLP_I218_LM:
        // case E1000_DEV_ID_PCH_LPTLP_I218_V:
        // case E1000_DEV_ID_PCH_I218_LM2:
        // case E1000_DEV_ID_PCH_I218_V2:
        // case E1000_DEV_ID_PCH_I218_LM3:		/* Dell UK */
        // case E1000_DEV_ID_PCH_I218_V3:
        // 	mac->type = e1000_pch_lpt;
        //      break;
        // case E1000_DEV_ID_PCH_SPT_I219_LM:		/* Dell US */
        // case E1000_DEV_ID_PCH_SPT_I219_V:
        // case E1000_DEV_ID_PCH_SPT_I219_LM2:
        // case E1000_DEV_ID_PCH_SPT_I219_V2:
        // case E1000_DEV_ID_PCH_LBG_I219_LM3:
        // case E1000_DEV_ID_PCH_SPT_I219_LM4:
        // case E1000_DEV_ID_PCH_SPT_I219_V4:
        // case E1000_DEV_ID_PCH_SPT_I219_LM5:
        // case E1000_DEV_ID_PCH_SPT_I219_V5:
        // 	mac->type = e1000_pch_spt;
        // 	break;

        self.hw.mac.mac_type = match self.hw.device_id as u32 {
            E1000_DEV_ID_82545EM_COPPER => MacType::Mac_82545,
            E1000_DEV_ID_82545EM_FIBER => MacType::Mac_82545,

            E1000_DEV_ID_PCH_LPT_I217_LM => MacType::Mac_pch_lpt,
            E1000_DEV_ID_PCH_LPT_I217_V => MacType::Mac_pch_lpt,
            E1000_DEV_ID_PCH_LPTLP_I218_LM => MacType::Mac_pch_lpt,
            E1000_DEV_ID_PCH_LPTLP_I218_V => MacType::Mac_pch_lpt,
            E1000_DEV_ID_PCH_I218_LM2 => MacType::Mac_pch_lpt,
            E1000_DEV_ID_PCH_I218_V2 => MacType::Mac_pch_lpt,
            E1000_DEV_ID_PCH_I218_LM3 => MacType::Mac_pch_lpt,
            E1000_DEV_ID_PCH_I218_V3 => MacType::Mac_pch_lpt,

            E1000_DEV_ID_PCH_SPT_I219_LM => MacType::Mac_pch_spt,
            E1000_DEV_ID_PCH_SPT_I219_V => MacType::Mac_pch_spt,
            E1000_DEV_ID_PCH_SPT_I219_LM2 => MacType::Mac_pch_spt,
            E1000_DEV_ID_PCH_SPT_I219_V2 => MacType::Mac_pch_spt,
            E1000_DEV_ID_PCH_LBG_I219_LM3 => MacType::Mac_pch_spt,
            E1000_DEV_ID_PCH_SPT_I219_LM4 => MacType::Mac_pch_spt,
            E1000_DEV_ID_PCH_SPT_I219_V4 => MacType::Mac_pch_spt,
            E1000_DEV_ID_PCH_SPT_I219_LM5 => MacType::Mac_pch_spt,
            E1000_DEV_ID_PCH_SPT_I219_V5 => MacType::Mac_pch_spt,
            _ => return Err("Hardware not supported".to_string()),
        };
        e1000_println!(
            "Device id: 0x{:x}, Mac type: {:?}",
            self.hw.device_id,
            self.hw.mac.mac_type
        );
        Ok(())
    }

    pub fn setup_shared_context(&mut self) -> AdResult {
        e1000_println!();
        // After calling identify_hardware() we know mac type
        try!(self.iflib_shared.setup(self.hw.mac.mac_type));
        Ok(())
    }

    pub fn allocate_pci_resources(&mut self) -> AdResult {
        e1000_println!();
        // struct adapter *adapter = iflib_get_softc(ctx);
        // device_t dev = iflib_get_dev(ctx);
        // int rid, val;

        // rid = PCIR_BAR(0);
        let mut rid = pcir_bar(0) as i32;

        // adapter->memory = bus_alloc_resource_any(dev, SYS_RES_MEMORY,
        //     &rid, RF_ACTIVE);
        // if (adapter->memory == NULL) {
        // 	device_printf(dev, "Unable to allocate bus resource: memory\n");
        // 	return (ENXIO);
        // }
        self.memory = self.dev
            .bus_alloc_resource_any(SYS_RES_MEMORY as i32, &mut rid, RF_ACTIVE);
        if self.memory.is_none() {
            return Err("Unable to allocate bus resource: memory".to_string());
        }

        // adapter->osdep.mem_bus_space_tag = rman_get_bustag(adapter->memory);
        self.osdep.mem_bus_space_tag = self.memory.as_ref().unwrap().rman_get_bustag();
        e1000_println!("mem_bus_space_tag {:x}", self.osdep.mem_bus_space_tag);

        // adapter->osdep.mem_bus_space_handle =
        //     rman_get_bushandle(adapter->memory);
        self.osdep.mem_bus_space_handle = self.memory.as_ref().unwrap().rman_get_bushandle();
        e1000_println!(
            "mem_bus_space_handle 0x{:x}",
            self.osdep.mem_bus_space_handle
        );

        let mem_ptr = self.osdep.mem_bus_space_handle as *mut u8;
        let len = self.memory.as_ref().unwrap().rman_get_size() as usize;
        self.hw.memory = MappedMemory::new(mem_ptr, len as usize);
        // e1000_println!("mem_bus_space_handle size 0x{:x}", len);

        // let slice = unsafe { kernel::slice::from_raw_parts_mut(mem_ptr, size as usize) };
        // adapter->hw.hw_addr = (u8 *)&adapter->osdep.mem_bus_space_handle;
        self.hw.hw_addr = unsafe { kernel::mem::transmute(&self.osdep.mem_bus_space_handle) };

        /* Only older adapters use IO mapping */

        if self.hw.mac.mac_type > MacType::Mac_82543 && self.hw.mac.mac_type < MacType::EM_MAC_MIN {
            // Our 82545 is here

            // 	/* Figure our where our IO BAR is ? */
            // 	for (rid = PCIR_BAR(0); rid < PCIR_CIS;) {
            // 		val = pci_read_config(dev, rid, 4);
            // 		if (EM_BAR_TYPE(val) == EM_BAR_TYPE_IO) {
            // 			adapter->io_rid = rid;
            // 			break;
            // 		}
            // 		rid += 4;
            // 		/* check for 64bit BAR */
            // 		if (EM_BAR_MEM_TYPE(val) == EM_BAR_MEM_TYPE_64BIT)
            // 			rid += 4;
            // 	}
            // 	if (rid >= PCIR_CIS) {
            // 		device_printf(dev, "Unable to locate IO BAR\n");
            // 		return (ENXIO);
            // 	}

            while rid < PCIR_CIS as i32 {
                let val = self.dev.pci_read_config(rid as u32, 4);
                if em_bar_type(val) == EM_BAR_TYPE_IO {
                    self.io_rid = rid;
                    break;
                }
                rid += 4;
                /* check for 64bit BAR */
                if em_bar_mem_type(val) == EM_BAR_MEM_TYPE_64BIT {
                    rid += 4;
                }
            }
            if rid >= PCIR_CIS as i32 {
                return Err("Unable to locate IO BAR".into());
            }

            // 	adapter->ioport = bus_alloc_resource_any(dev,
            // 	    SYS_RES_IOPORT, &adapter->io_rid, RF_ACTIVE);
            // 	if (adapter->ioport == NULL) {
            // 		device_printf(dev, "Unable to allocate bus resource: "
            // 		    "ioport\n");
            // 		return (ENXIO);
            // 	}

            if let Some(r) =
                self.dev
                    .bus_alloc_resource_any(SYS_RES_IOPORT as i32, &mut self.io_rid, RF_ACTIVE)
            {
                self.ioport = Some(r);
            } else {
                return Err("Unable to allocate bus resource: ioport".into());
            }

            // 	adapter->hw.io_base = 0;
            // 	adapter->osdep.io_bus_space_tag =
            // 	    rman_get_bustag(adapter->ioport);
            // 	adapter->osdep.io_bus_space_handle =
            // 	    rman_get_bushandle(adapter->ioport);
            // }
            // adapter->hw.back = &adapter->osdep;

            self.hw.io_base = 0;
            self.osdep.io_bus_space_tag = self.ioport.as_ref().unwrap().rman_get_bustag();
            self.osdep.io_bus_space_handle = self.ioport.as_ref().unwrap().rman_get_bushandle();

            // e1000_println!("io_bus_space_handle 0x{:x}", self.osdep.io_bus_space_handle);

            // let size = self.ioport.as_ref().unwrap().rman_get_size();
            // e1000_println!("io_bus_space_handle size 0x{:x}", size);
        }
        // self.hw.back = self.osdep;
        Ok(())
    }
    pub fn setup_init_functions(&mut self, init_device: bool) -> AdResult {
        e1000_println!();

        try!(self.set_mac_type());

        /*
         * Init function pointers to generic implementations. We do this first
         * allowing a driver module to override it afterward.
         */
        self.hw.phy.ops.init_generic();
        self.hw.mac.ops.init_generic();
        self.hw.mbx.ops.init_generic();
        self.hw.nvm.ops.init_generic();

        /*
         * Set up the init function pointers. These are functions within the
         * adapter family file that sets up function pointers for the rest of
         * the functions in that family.
         */
        let macs_82540 = [
            MacType::Mac_82540,
            MacType::Mac_82545,
            MacType::Mac_82545_rev_3,
            MacType::Mac_82546,
            MacType::Mac_82546_rev_3,
        ];
        let macs_ich8lan = [
            MacType::Mac_ich8lan,
            MacType::Mac_ich9lan,
            MacType::Mac_ich10lan,
            MacType::Mac_pchlan,
            MacType::Mac_pch2lan,
            MacType::Mac_pch_lpt,
            MacType::Mac_pch_spt,
            MacType::Mac_pch_cnp,
        ];

        if self.is_macs(&macs_82540) {
            try!(e1000_82540::init_function_pointers(self));
        } else if self.is_macs(&macs_ich8lan) {
            try!(e1000_ich8lan::init_function_pointers(self));
        } else {
            return Err("Unsupported hardware".to_string());
        }

        if init_device {
            if let Some(f) = self.hw.mac.ops.init_params {
                try!(f(self));
                e1000_println!("Init mac params done");
            }
            if let Some(f) = self.hw.nvm.ops.init_params {
                try!(f(self));
                e1000_println!("Init nvm params done");
            }
            if let Some(f) = self.hw.phy.ops.init_params {
                try!(f(self));
                e1000_println!("Init phy params done");
            }
            if let Some(f) = self.hw.mbx.ops.init_params {
                try!(f(self));
                e1000_println!("Init mbx params done");
            }
        }
        Ok(())
    }
    pub fn setup_msix(&mut self) -> AdResult {
        // e1000_println!();
        // if (adapter->hw.mac.type == e1000_82574) {
        // 	em_enable_vectors_82574(ctx);
        // }
        if self.hw.mac.mac_type == MacType::Mac_82574 {
            Err("MSIX not implemented".to_string())
        } else {
            e1000_println!("This hardware does not have MSIX (only for 82574)");
            Ok(())
        }
    }
    pub fn check_reset_block(&mut self) -> Result<bool, String> {
        e1000_verbose_println!();
        self.hw
            .phy
            .ops
            .check_reset_block
            .ok_or("No function".to_string())
            .and_then(|f| f(self))
    }
    pub fn validate_nvm_checksum(&mut self) -> AdResult {
        e1000_println!();
        self.hw
            .nvm
            .ops
            .validate
            .ok_or("No function".to_string())
            .and_then(|f| f(self))
    }
    pub fn init_manageability(&mut self) {
        e1000_println!();
        // /* A shared code workaround */
        // #define E1000_82542_MANC2H E1000_MANC2H
        // 	if (adapter->has_manage) {
        // 		int manc2h = E1000_READ_REG(&adapter->hw, E1000_MANC2H);
        // 		int manc = E1000_READ_REG(&adapter->hw, E1000_MANC);

        // 		/* disable hardware interception of ARP */
        // 		manc &= ~(E1000_MANC_ARP_EN);

        // 		/* enable receiving management packets to the host */
        // 		manc |= E1000_MANC_EN_MNG2HOST;
        // #define E1000_MNG2HOST_PORT_623 (1 << 5)
        // #define E1000_MNG2HOST_PORT_664 (1 << 6)
        // 		manc2h |= E1000_MNG2HOST_PORT_623;
        // 		manc2h |= E1000_MNG2HOST_PORT_664;
        // 		E1000_WRITE_REG(&adapter->hw, E1000_MANC2H, manc2h);
        // 		E1000_WRITE_REG(&adapter->hw, E1000_MANC, manc);
        // 	}
        if self.has_manage {
            e1000_println!("Adapter has manage");
            let mut manc2h = do_read_register(self, E1000_MANC2H);
            let mut manc = do_read_register(self, E1000_MANC);
            manc &= !E1000_MANC_ARP_EN;
            manc |= E1000_MANC_EN_MNG2HOST;
            manc2h |= 1 << 5;
            manc2h |= 1 << 6;
            do_write_register(self, E1000_MANC2H, manc2h);
            do_write_register(self, E1000_MANC, manc);
        }
    }
    pub fn initialize_transmit_unit(&mut self) {
        e1000_println!();
        // struct adapter *adapter = iflib_get_softc(ctx);
        // if_softc_ctx_t scctx = adapter->shared;
        // struct em_tx_queue *que;
        // struct tx_ring	*txr;
        // struct e1000_hw	*hw = &adapter->hw;
        // u32 tctl, txdctl = 0, tarc, tipg = 0;

        // INIT_DEBUGOUT("em_initialize_transmit_unit: begin");

        // for (int i = 0; i < adapter->tx_num_queues; i++, txr++) {
        // 	u64 bus_addr;
        // 	caddr_t offp, endp;

        for i in 0..self.iflib_shared.isc_nrxqsets as usize {
            // que = &adapter->tx_queues[i];
            // txr = &que->txr;
            // bus_addr = txr->tx_paddr;

            // /* Clear checksum offload context. */
            // offp = (caddr_t)&txr->csum_flags;
            // endp = (caddr_t)(txr + 1);
            // bzero(offp, endp - offp);

            self.tx_queues[i].txr.clear_checksum();
            let bus_addr = self.tx_queues[i].txr.tx_paddr;

            // 	/* Base and Len of TX Ring */
            // 	E1000_WRITE_REG(hw, E1000_TDLEN(i),
            // 	    scctx->isc_ntxd[0] * sizeof(struct e1000_tx_desc));
            // 	E1000_WRITE_REG(hw, E1000_TDBAH(i),
            // 	    (u32)(bus_addr >> 32));
            // 	E1000_WRITE_REG(hw, E1000_TDBAL(i),
            // 	    (u32)bus_addr);
            // 	/* Init the HEAD/TAIL indices */
            // 	E1000_WRITE_REG(hw, E1000_TDT(i), 0);
            // 	E1000_WRITE_REG(hw, E1000_TDH(i), 0);

            do_write_register(
                self,
                E1000_TDLEN(i),
                self.iflib_shared.isc_ntxd[0] as u32
                    * kernel::mem::size_of::<e1000_tx_desc>() as u32,
            );
            do_write_register(self, E1000_TDBAH(i), (bus_addr >> 32) as u32);
            do_write_register(self, E1000_TDBAL(i), bus_addr as u32);
            do_write_register(self, E1000_TDT(i), 0);
            do_write_register(self, E1000_TDH(i), 0);

            // 	HW_DEBUGOUT2("Base = %x, Length = %x\n",
            // 	    E1000_READ_REG(&adapter->hw, E1000_TDBAL(i)),
            // 	    E1000_READ_REG(&adapter->hw, E1000_TDLEN(i)));

            // Base = 9e04000, Length = 4000

            e1000_println!(
                "Base = 0x{:x}, Length = 0x{:x}",
                do_read_register(self, E1000_TDBAL(i)),
                do_read_register(self, E1000_TDLEN(i)),
            );
            // 	txdctl = 0; /* clear txdctl */
            // 	txdctl |= 0x1f; /* PTHRESH */
            // 	txdctl |= 1 << 8; /* HTHRESH */
            // 	txdctl |= 1 << 16;/* WTHRESH */
            // 	txdctl |= 1 << 22; /* Reserved bit 22 must always be 1 */
            // 	txdctl |= E1000_TXDCTL_GRAN;
            // 	txdctl |= 1 << 25; /* LWTHRESH */
            let mut txdctl = 0;
            txdctl |= 0x1f; /* PTHRESH */
            txdctl |= 1 << 8; /* HTHRESH */
            txdctl |= 1 << 16; /* WTHRESH */
            txdctl |= 1 << 22; /* Reserved bit 22 must always be 1 */
            txdctl |= E1000_TXDCTL_GRAN;
            txdctl |= 1 << 25; /* LWTHRESH */

            // 	E1000_WRITE_REG(hw, E1000_TXDCTL(i), txdctl);
            do_write_register(self, E1000_TXDCTL(i), txdctl);
            // }
        }

        // /* Set the default values for the Tx Inter Packet Gap timer */
        // switch (adapter->hw.mac.type) {
        // case e1000_80003es2lan:
        // 	tipg = DEFAULT_82543_TIPG_IPGR1;
        // 	tipg |= DEFAULT_80003ES2LAN_TIPG_IPGR2 <<
        // 	    E1000_TIPG_IPGR2_SHIFT;
        // 	break;
        // case e1000_82542:
        // 	tipg = DEFAULT_82542_TIPG_IPGT;
        // 	tipg |= DEFAULT_82542_TIPG_IPGR1 << E1000_TIPG_IPGR1_SHIFT;
        // 	tipg |= DEFAULT_82542_TIPG_IPGR2 << E1000_TIPG_IPGR2_SHIFT;
        // 	break;
        // default:
        // 	if ((adapter->hw.phy.media_type == e1000_media_type_fiber) ||
        // 	    (adapter->hw.phy.media_type ==
        // 	    e1000_media_type_internal_serdes))
        // 		tipg = DEFAULT_82543_TIPG_IPGT_FIBER;
        // 	else
        // 		tipg = DEFAULT_82543_TIPG_IPGT_COPPER;
        // 	tipg |= DEFAULT_82543_TIPG_IPGR1 << E1000_TIPG_IPGR1_SHIFT;
        // 	tipg |= DEFAULT_82543_TIPG_IPGR2 << E1000_TIPG_IPGR2_SHIFT;
        // }
        let tipg = match self.hw.mac.mac_type {
            MacType::Mac_80003es2lan => panic!("Unsupported hardware"),
            MacType::Mac_82542 => panic!("Unsupported hardware"),
            _ => {
                let mut t = {
                    if self.hw.phy.media_type == MediaType::Fiber
                        || self.hw.phy.media_type == MediaType::InternalSerdes
                    {
                        // DEFAULT_82543_TIPG_IPGT_FIBER
                        panic!("Unsupported hardware");
                    } else {
                        DEFAULT_82543_TIPG_IPGT_COPPER
                    }
                };
                t |= DEFAULT_82543_TIPG_IPGR1 << E1000_TIPG_IPGR1_SHIFT;
                t |= DEFAULT_82543_TIPG_IPGR2 << E1000_TIPG_IPGR2_SHIFT;
                t
            }
        };

        // E1000_WRITE_REG(&adapter->hw, E1000_TIPG, tipg);
        // E1000_WRITE_REG(&adapter->hw, E1000_TIDV, adapter->tx_int_delay.value);
        do_write_register(self, E1000_TIPG, tipg);
        do_write_register(self, E1000_TIDV, self.tx_int_delay.value as u32);

        // if(adapter->hw.mac.type >= e1000_82540)
        // 	E1000_WRITE_REG(&adapter->hw, E1000_TADV,
        // 	    adapter->tx_abs_int_delay.value);
        if self.hw.mac.mac_type >= MacType::Mac_82540 {
            do_write_register(self, E1000_TADV, self.tx_abs_int_delay.value as u32);
        }

        // if ((adapter->hw.mac.type == e1000_82571) ||
        //     (adapter->hw.mac.type == e1000_82572)) {
        // 	tarc = E1000_READ_REG(&adapter->hw, E1000_TARC(0));
        // 	tarc |= TARC_SPEED_MODE_BIT;
        // 	E1000_WRITE_REG(&adapter->hw, E1000_TARC(0), tarc);
        // } else if (adapter->hw.mac.type == e1000_80003es2lan) {
        // 	/* errata: program both queues to unweighted RR */
        // 	tarc = E1000_READ_REG(&adapter->hw, E1000_TARC(0));
        // 	tarc |= 1;
        // 	E1000_WRITE_REG(&adapter->hw, E1000_TARC(0), tarc);
        // 	tarc = E1000_READ_REG(&adapter->hw, E1000_TARC(1));
        // 	tarc |= 1;
        // 	E1000_WRITE_REG(&adapter->hw, E1000_TARC(1), tarc);
        // } else if (adapter->hw.mac.type == e1000_82574) {
        // 	tarc = E1000_READ_REG(&adapter->hw, E1000_TARC(0));
        // 	tarc |= TARC_ERRATA_BIT;
        // 	if ( adapter->tx_num_queues > 1) {
        // 		tarc |= (TARC_COMPENSATION_MODE | TARC_MQ_FIX);
        // 		E1000_WRITE_REG(&adapter->hw, E1000_TARC(0), tarc);
        // 		E1000_WRITE_REG(&adapter->hw, E1000_TARC(1), tarc);
        // 	} else
        // 		E1000_WRITE_REG(&adapter->hw, E1000_TARC(0), tarc);
        // }
        if self.hw.mac.mac_type == MacType::Mac_82571 || self.hw.mac.mac_type == MacType::Mac_82572
        {
            panic!("Unsupported hardware");
        } else if self.hw.mac.mac_type == MacType::Mac_80003es2lan {
            panic!("Unsupported hardware");
        } else if self.hw.mac.mac_type == MacType::Mac_82574 {
            panic!("Unsupported hardware");
        }

        // if (adapter->tx_int_delay.value > 0)
        // 	adapter->txd_cmd |= E1000_TXD_CMD_IDE;
        if self.tx_int_delay.value > 0 {
            self.txd_cmd |= E1000_TXD_CMD_IDE;
        }

        // /* Program the Transmit Control Register */
        // tctl = E1000_READ_REG(&adapter->hw, E1000_TCTL);
        // tctl &= ~E1000_TCTL_CT;
        // tctl |= (E1000_TCTL_PSP | E1000_TCTL_RTLC | E1000_TCTL_EN |
        // 	   (E1000_COLLISION_THRESHOLD << E1000_CT_SHIFT));
        let mut tctl = do_read_register(self, E1000_TCTL);
        tctl &= !E1000_TCTL_CT;
        tctl |= E1000_TCTL_PSP | E1000_TCTL_RTLC | E1000_TCTL_EN
            | (E1000_COLLISION_THRESHOLD << E1000_CT_SHIFT);

        // if (adapter->hw.mac.type >= e1000_82571)
        // 	tctl |= E1000_TCTL_MULR;
        if self.hw.mac.mac_type == MacType::Mac_82571 {
            panic!("Unsupported hardware");
        }

        // /* This write will effectively turn on the transmit unit. */
        // E1000_WRITE_REG(&adapter->hw, E1000_TCTL, tctl);
        do_write_register(self, E1000_TCTL, tctl);

        // /* SPT and KBL errata workarounds */
        // if (hw->mac.type == e1000_pch_spt) {
        // 	u32 reg;
        // 	reg = E1000_READ_REG(hw, E1000_IOSFPC);
        // 	reg |= E1000_RCTL_RDMTS_HEX;
        // 	E1000_WRITE_REG(hw, E1000_IOSFPC, reg);
        // 	/* i218-i219 Specification Update 1.5.4.5 */
        // 	reg = E1000_READ_REG(hw, E1000_TARC(0));
        // 	reg &= ~E1000_TARC0_CB_MULTIQ_3_REQ;
        // 	reg |= E1000_TARC0_CB_MULTIQ_2_REQ;
        // 	E1000_WRITE_REG(hw, E1000_TARC(0), reg);
        // }
        /* SPT and KBL errata workarounds */
        if self.hw.mac.mac_type == MacType::Mac_pch_spt {
            let mut reg: u32 = do_read_register(self, E1000_IOSFPC);
            reg |= E1000_RCTL_RDMTS_HEX;
            do_write_register(self, E1000_IOSFPC, reg);
            /* i218-i219 Specification Update 1.5.4.5 */
            reg = do_read_register(self, E1000_TARC(0));
            reg &= !E1000_TARC0_CB_MULTIQ_3_REQ;
            reg |= E1000_TARC0_CB_MULTIQ_2_REQ;
            do_write_register(self, E1000_TARC(0), reg);
        }
    }
    pub fn initialize_receive_unit(&mut self) -> AdResult {
        e1000_println!();
        // struct adapter *adapter = iflib_get_softc(ctx);
        // 	if_softc_ctx_t scctx = adapter->shared;
        // 	struct ifnet *ifp = iflib_get_ifp(ctx);
        // 	struct e1000_hw	*hw = &adapter->hw;
        // 	struct em_rx_queue *que;
        // 	int i;
        // 	u32 rctl, rxcsum, rfctl;
        let mut rctl: u32;
        let mut rxcsum: u32;
        let mut rfctl: u32;
        // 	INIT_DEBUGOUT("em_initialize_receive_units: begin");

        /*
         * Make sure receives are disabled while setting
         * up the descriptor ring
         */
        // 	rctl = E1000_READ_REG(hw, E1000_RCTL);
        // 	/* Do not disable if ever enabled on this hardware */
        // 	if ((hw->mac.type != e1000_82574) && (hw->mac.type != e1000_82583))
        // 		E1000_WRITE_REG(hw, E1000_RCTL, rctl & ~E1000_RCTL_EN);
        rctl = do_read_register(self, E1000_RCTL);
        if self.is_not_macs(&[MacType::Mac_82574, MacType::Mac_82583]) {
            do_write_register(self, E1000_RCTL, rctl & !E1000_RCTL_EN);
        }

        /* Setup the Receive Control Register */
        // 	rctl &= ~(3 << E1000_RCTL_MO_SHIFT);
        // 	rctl |= E1000_RCTL_EN | E1000_RCTL_BAM |
        // 	    E1000_RCTL_LBM_NO | E1000_RCTL_RDMTS_HALF |
        // 	    (hw->mac.mc_filter_type << E1000_RCTL_MO_SHIFT);
        rctl &= !(3 << E1000_RCTL_MO_SHIFT);
        rctl |= E1000_RCTL_EN | E1000_RCTL_BAM | E1000_RCTL_LBM_NO | E1000_RCTL_RDMTS_HALF
            | (self.hw.mac.mc_filter_type << E1000_RCTL_MO_SHIFT);

        /* Do not store bad packets */
        // 	rctl &= ~E1000_RCTL_SBP;
        rctl &= !E1000_RCTL_SBP;

        /* Enable Long Packet receive */
        // 	if (if_getmtu(ifp) > ETHERMTU)
        // 		rctl |= E1000_RCTL_LPE;
        // 	else
        // 		rctl &= ~E1000_RCTL_LPE;
        if self.ifnet.mtu() > ETHERMTU as i32 {
            rctl |= E1000_RCTL_LPE;
        } else {
            rctl &= !E1000_RCTL_LPE;
        }

        //      /* Strip the CRC */
        // 	if (!em_disable_crc_stripping)
        // 		rctl |= E1000_RCTL_SECRC;

        // ^^^ No sysctls yet - default to 0

        // 	if (adapter->hw.mac.type >= e1000_82540) {
        // 		E1000_WRITE_REG(&adapter->hw, E1000_RADV,
        // 			    adapter->rx_abs_int_delay.value);

        // 		/*
        // 		 * Set the interrupt throttling rate. Value is calculated
        // 		 * as DEFAULT_ITR = 1/(MAX_INTS_PER_SEC * 256ns)
        // 		 */
        // 		E1000_WRITE_REG(hw, E1000_ITR, DEFAULT_ITR);
        // 	}

        if self.hw.mac.mac_type >= MacType::Mac_82540 {
            do_write_register(self, E1000_RADV, self.rx_abs_int_delay.value);
            do_write_register(self, E1000_ITR, DEFAULT_ITR);
        }

        // 	E1000_WRITE_REG(&adapter->hw, E1000_RDTR,
        // 	    adapter->rx_int_delay.value);
        do_write_register(self, E1000_RDTR, self.rx_int_delay.value);

        /* Use extended rx descriptor formats */
        // 	rfctl = E1000_READ_REG(hw, E1000_RFCTL);
        // 	rfctl |= E1000_RFCTL_EXTEN;
        rfctl = do_read_register(self, E1000_RFCTL);
        rfctl |= E1000_RFCTL_EXTEN;

        /*
         * When using MSIX interrupts we need to throttle
         * using the EITR register (82574 only)
         */
        // 	if (hw->mac.type == e1000_82574) {
        // 		for (int i = 0; i < 4; i++)
        // 			E1000_WRITE_REG(hw, E1000_EITR_82574(i),
        // 			    DEFAULT_ITR);
        // 		/* Disable accelerated acknowledge */
        // 		rfctl |= E1000_RFCTL_ACK_DIS;
        // 	}
        // 	E1000_WRITE_REG(hw, E1000_RFCTL, rfctl);
        if self.is_mac(MacType::Mac_82574) {
            incomplete_return!();
        }
        do_write_register(self, E1000_RFCTL, rfctl);

        // 	rxcsum = E1000_READ_REG(hw, E1000_RXCSUM);
        rxcsum = do_read_register(self, E1000_RXCSUM);

        // 	if (if_getcapenable(ifp) & IFCAP_RXCSUM &&
        // 	    adapter->hw.mac.type >= e1000_82543) {
        // 		if (adapter->tx_num_queues > 1) {
        // 			if (adapter->hw.mac.type >= igb_mac_min) {
        // 				rxcsum |= E1000_RXCSUM_PCSD;
        // 				if (hw->mac.type != e1000_82575)
        // 					rxcsum |= E1000_RXCSUM_CRCOFL;
        // 			} else
        // 				rxcsum |= E1000_RXCSUM_TUOFL |
        // 					E1000_RXCSUM_IPOFL |
        // 					E1000_RXCSUM_PCSD;
        // 		} else {
        // 			if (adapter->hw.mac.type >= igb_mac_min)
        // 				rxcsum |= E1000_RXCSUM_IPPCSE;
        // 			else
        // 				rxcsum |= E1000_RXCSUM_TUOFL | E1000_RXCSUM_IPOFL;
        // 			if (adapter->hw.mac.type > e1000_82575)
        // 				rxcsum |= E1000_RXCSUM_CRCOFL;
        // 		}
        // 	} else
        // 		rxcsum &= ~E1000_RXCSUM_TUOFL;
        if self.ifnet.capenable() & IFCAP_RXCSUM != 0 && self.hw.mac.mac_type >= MacType::Mac_82543
        {
            if self.tx_num_queues() > 1 {
                incomplete_return!();
            } else {
                if self.hw.mac.mac_type >= MacType::IGB_MAC_MIN {
                    rxcsum |= E1000_RXCSUM_IPPCSE;
                } else {
                    rxcsum |= E1000_RXCSUM_TUOFL | E1000_RXCSUM_IPOFL;
                }
                if self.hw.mac.mac_type > MacType::Mac_82575 {
                    rxcsum |= E1000_RXCSUM_CRCOFL;
                }
            }
        } else {
            rxcsum &= !E1000_RXCSUM_TUOFL;
        }

        // E1000_WRITE_REG(hw, E1000_RXCSUM, rxcsum);
        do_write_register(self, E1000_RXCSUM, rxcsum);

        // 	if (adapter->rx_num_queues > 1) {
        // 		if (adapter->hw.mac.type >= igb_mac_min)
        // 			igb_initialize_rss_mapping(adapter);
        // 		else
        // 			em_initialize_rss_mapping(adapter);
        // 	}
        if self.rx_num_queues() > 1 {
            incomplete_return!();
        }

        /*
         * XXX TEMPORARY WORKAROUND: on some systems with 82573
         * long latencies are observed, like Lenovo X60. This
         * change eliminates the problem, but since having positive
         * values in RDTR is a known source of problems on other
         * platforms another solution is being sought.
         */
        // 	if (hw->mac.type == e1000_82573)
        // 		E1000_WRITE_REG(hw, E1000_RDTR, 0x20);
        if self.is_mac(MacType::Mac_82573) {
            incomplete_return!();
        }
        // 	for (i = 0, que = adapter->rx_queues; i < adapter->rx_num_queues; i++, que++) {
        // 		struct rx_ring *rxr = &que->rxr;
        // 		/* Setup the Base and Length of the Rx Descriptor Ring */
        // 		u64 bus_addr = rxr->rx_paddr;
        // #if 0
        // 		u32 rdt = adapter->rx_num_queues -1;  /* default */
        // #endif

        // 		E1000_WRITE_REG(hw, E1000_RDLEN(i),
        // 		    scctx->isc_nrxd[0] * sizeof(union e1000_rx_desc_extended));
        // 		E1000_WRITE_REG(hw, E1000_RDBAH(i), (u32)(bus_addr >> 32));
        // 		E1000_WRITE_REG(hw, E1000_RDBAL(i), (u32)bus_addr);
        // 		/* Setup the Head and Tail Descriptor Pointers */
        // 		E1000_WRITE_REG(hw, E1000_RDH(i), 0);
        // 		E1000_WRITE_REG(hw, E1000_RDT(i), 0);
        // 	}
        for (i, rxqueue) in &mut self.rx_queues.iter().enumerate() {
            let rxq: &RxQueue = rxqueue;
            let bus_addr = rxq.rxr.rx_paddr;

            do_write_register(
                self,
                E1000_RDLEN(i),
                self.iflib_shared.isc_nrxd[0] as u32
                    * kernel::mem::size_of::<e1000_rx_desc_extended>() as u32,
            );
            do_write_register(self, E1000_RDBAH(i), (bus_addr >> 32) as u32);
            do_write_register(self, E1000_RDBAL(i), bus_addr as u32);
            do_write_register(self, E1000_RDH(i), 0);
            do_write_register(self, E1000_RDT(i), 0);
        }

        /*
         * Set PTHRESH for improved jumbo performance
         * According to 10.2.5.11 of Intel 82574 Datasheet,
         * RXDCTL(1) is written whenever RXDCTL(0) is written.
         * Only write to RXDCTL(1) if there is a need for different
         * settings.
         */
        // 	if (((adapter->hw.mac.type == e1000_ich9lan) ||
        // 	    (adapter->hw.mac.type == e1000_pch2lan) ||
        // 	    (adapter->hw.mac.type == e1000_ich10lan)) &&
        // 	    (if_getmtu(ifp) > ETHERMTU)) {
        // 		u32 rxdctl = E1000_READ_REG(hw, E1000_RXDCTL(0));
        // 		E1000_WRITE_REG(hw, E1000_RXDCTL(0), rxdctl | 3);
        if self.is_macs(&[
            MacType::Mac_ich9lan,
            MacType::Mac_pch2lan,
            MacType::Mac_ich10lan,
        ]) && self.ifnet.mtu() > ETHERMTU as i32
        {
            incomplete_return!();
        }
        // 	} else if (adapter->hw.mac.type == e1000_82574) {
        // 		for (int i = 0; i < adapter->rx_num_queues; i++) {
        // 			u32 rxdctl = E1000_READ_REG(hw, E1000_RXDCTL(i));
        // 			rxdctl |= 0x20; /* PTHRESH */
        // 			rxdctl |= 4 << 8; /* HTHRESH */
        // 			rxdctl |= 4 << 16;/* WTHRESH */
        // 			rxdctl |= 1 << 24; /* Switch to granularity */
        // 			E1000_WRITE_REG(hw, E1000_RXDCTL(i), rxdctl);
        // 		}
        else if self.is_mac(MacType::Mac_82574) {
            incomplete_return!();
        }
        // 	} else if (adapter->hw.mac.type >= igb_mac_min) {
        // 		u32 psize, srrctl = 0;

        // 		if (if_getmtu(ifp) > ETHERMTU) {
        // 			/* Set maximum packet len */
        // 			if (adapter->rx_mbuf_sz <= 4096) {
        // 				srrctl |= 4096 >> E1000_SRRCTL_BSIZEPKT_SHIFT;
        // 				rctl |= E1000_RCTL_SZ_4096 | E1000_RCTL_BSEX;
        // 			} else if (adapter->rx_mbuf_sz > 4096) {
        // 				srrctl |= 8192 >> E1000_SRRCTL_BSIZEPKT_SHIFT;
        // 				rctl |= E1000_RCTL_SZ_8192 | E1000_RCTL_BSEX;
        // 			}
        // 			psize = scctx->isc_max_frame_size;
        // 			/* are we on a vlan? */
        // 			if (ifp->if_vlantrunk != NULL)
        // 				psize += VLAN_TAG_SIZE;
        // 			E1000_WRITE_REG(&adapter->hw, E1000_RLPML, psize);
        // 		} else {
        // 			srrctl |= 2048 >> E1000_SRRCTL_BSIZEPKT_SHIFT;
        // 			rctl |= E1000_RCTL_SZ_2048;
        // 		}

        // 		/*
        // 		 * If TX flow control is disabled and there's >1 queue defined,
        // 		 * enable DROP.
        // 		 *
        // 		 * This drops frames rather than hanging the RX MAC for all queues.
        // 		 */
        // 		if ((adapter->rx_num_queues > 1) &&
        // 		    (adapter->fc == e1000_fc_none ||
        // 		     adapter->fc == e1000_fc_rx_pause)) {
        // 			srrctl |= E1000_SRRCTL_DROP_EN;
        // 		}
        // 			/* Setup the Base and Length of the Rx Descriptor Rings */
        // 		for (i = 0, que = adapter->rx_queues; i < adapter->rx_num_queues; i++, que++) {
        // 			struct rx_ring *rxr = &que->rxr;
        // 			u64 bus_addr = rxr->rx_paddr;
        // 			u32 rxdctl;

        // #ifdef notyet
        // 			/* Configure for header split? -- ignore for now */
        // 			rxr->hdr_split = igb_header_split;
        // #else
        // 			srrctl |= E1000_SRRCTL_DESCTYPE_ADV_ONEBUF;
        // #endif

        // 			E1000_WRITE_REG(hw, E1000_RDLEN(i),
        // 					scctx->isc_nrxd[0] * sizeof(struct e1000_rx_desc));
        // 			E1000_WRITE_REG(hw, E1000_RDBAH(i),
        // 					(uint32_t)(bus_addr >> 32));
        // 			E1000_WRITE_REG(hw, E1000_RDBAL(i),
        // 					(uint32_t)bus_addr);
        // 			E1000_WRITE_REG(hw, E1000_SRRCTL(i), srrctl);
        // 			/* Enable this Queue */
        // 			rxdctl = E1000_READ_REG(hw, E1000_RXDCTL(i));
        // 			rxdctl |= E1000_RXDCTL_QUEUE_ENABLE;
        // 			rxdctl &= 0xFFF00000;
        // 			rxdctl |= IGB_RX_PTHRESH;
        // 			rxdctl |= IGB_RX_HTHRESH << 8;
        // 			rxdctl |= IGB_RX_WTHRESH << 16;
        // 			E1000_WRITE_REG(hw, E1000_RXDCTL(i), rxdctl);
        // 		}
        else if self.hw.mac.mac_type >= MacType::IGB_MAC_MIN {
            incomplete_return!();
        }
        // 	} else if (adapter->hw.mac.type >= e1000_pch2lan) {
        // 		if (if_getmtu(ifp) > ETHERMTU)
        // 			e1000_lv_jumbo_workaround_ich8lan(hw, TRUE);
        // 		else
        // 			e1000_lv_jumbo_workaround_ich8lan(hw, FALSE);
        // 	}
        else if self.hw.mac.mac_type >= MacType::Mac_pch2lan {
            // Dell hardware is here
            if self.ifnet.mtu() > ETHERMTU as i32 {
                try!(e1000_ich8lan::lv_jumbo_workaround(self, true))
            } else {
                try!(e1000_ich8lan::lv_jumbo_workaround(self, false))
            }
        }

        /* Make sure VLAN Filters are off */
        // 	rctl &= ~E1000_RCTL_VFE;
        rctl &= !E1000_RCTL_VFE;

        // if (adapter->hw.mac.type < igb_mac_min) {
        //     if (adapter->rx_mbuf_sz == MCLBYTES)
        // 	rctl |= E1000_RCTL_SZ_2048;
        //     else if (adapter->rx_mbuf_sz == MJUMPAGESIZE)
        // 	rctl |= E1000_RCTL_SZ_4096 | E1000_RCTL_BSEX;
        //     else if (adapter->rx_mbuf_sz > MJUMPAGESIZE)
        // 	rctl |= E1000_RCTL_SZ_8192 | E1000_RCTL_BSEX;

        //     /* ensure we clear use DTYPE of 00 here */
        //     rctl &= ~0x00000C00;
        // }
        if self.hw.mac.mac_type < MacType::IGB_MAC_MIN {
            match self.rx_mbuf_sz {
                kernel::sys::iflib_sys::MCLBYTES => {
                    rctl |= E1000_RCTL_SZ_2048;
                }
                kernel::sys::iflib_sys::MJUMPAGESIZE => {
                    incomplete!();
                }
                x if x > kernel::sys::iflib_sys::MJUMPAGESIZE => {
                    incomplete!();
                }
                _ => e1000_println!("Invalid rx mbuf size"),
            }
            rctl &= !0x00000C00;
        }

        /* Write out the settings */
        // E1000_WRITE_REG(hw, E1000_RCTL, rctl);
        do_write_register(self, E1000_RCTL, rctl);
        Ok(())
    }
    pub fn multi_set(&mut self) -> AdResult {
        e1000_println!();
        // struct adapter *adapter = iflib_get_softc(ctx);
        // struct ifnet *ifp = iflib_get_ifp(ctx);
        // u32 reg_rctl = 0;
        // u8  *mta; /* Multicast array memory */
        // int mcnt = 0;
        let mut reg_rctl: u32;

        // IOCTL_DEBUGOUT("em_set_multi: begin");

        // mta = adapter->mta;
        // bzero(mta, sizeof(u8) * ETH_ADDR_LEN * MAX_NUM_MULTICAST_ADDRESSES);
        for item in &mut self.mta.iter_mut() {
            *item = 0;
        }

        // if (adapter->hw.mac.type == e1000_82542 &&
        //     adapter->hw.revision_id == E1000_REVISION_2) {
        // 	reg_rctl = E1000_READ_REG(&adapter->hw, E1000_RCTL);
        // 	if (adapter->hw.bus.pci_cmd_word & CMD_MEM_WRT_INVALIDATE)
        // 		e1000_pci_clear_mwi(&adapter->hw);
        // 	reg_rctl |= E1000_RCTL_RST;
        // 	E1000_WRITE_REG(&adapter->hw, E1000_RCTL, reg_rctl);
        // 	msec_delay(5);
        // }
        if self.is_mac(MacType::Mac_82542) && self.hw.revision_id == E1000_REVISION_2 as u8 {
            return Err("Unsupported hardware".to_string());
        }

        // if_multiaddr_array(ifp, mta, &mcnt, MAX_NUM_MULTICAST_ADDRESSES);
        let mut mcnt: u32 = 0;
        self.ifnet.multiaddr_array(
            self.mta.as_mut_ptr(),
            &mut mcnt,
            MAX_NUM_MULTICAST_ADDRESSES,
        );
        // e1000_println!("mcnt returned {}", mcnt);
        // e1000_println!("self.mta {:?}", self.mta);

        // > mta = 01:00:5e:00:00:01

        // if (mcnt >= MAX_NUM_MULTICAST_ADDRESSES) {
        // 	reg_rctl = E1000_READ_REG(&adapter->hw, E1000_RCTL);
        // 	reg_rctl |= E1000_RCTL_MPE;
        // 	E1000_WRITE_REG(&adapter->hw, E1000_RCTL, reg_rctl);
        // } else
        // 	e1000_update_mc_addr_list(&adapter->hw, mta, mcnt);
        if mcnt >= MAX_NUM_MULTICAST_ADDRESSES {
            reg_rctl = do_read_register(self, E1000_RCTL);
            reg_rctl |= E1000_RCTL_MPE;
            do_write_register(self, E1000_RCTL, reg_rctl);
        } else {
            self.update_mc_addr_list(mcnt);
        }
        // if (adapter->hw.mac.type == e1000_82542 &&
        //     adapter->hw.revision_id == E1000_REVISION_2) {
        // 	reg_rctl = E1000_READ_REG(&adapter->hw, E1000_RCTL);
        // 	reg_rctl &= ~E1000_RCTL_RST;
        // 	E1000_WRITE_REG(&adapter->hw, E1000_RCTL, reg_rctl);
        // 	msec_delay(5);
        // 	if (adapter->hw.bus.pci_cmd_word & CMD_MEM_WRT_INVALIDATE)
        // 		e1000_pci_set_mwi(&adapter->hw);
        // }
        if self.is_mac(MacType::Mac_82542) && self.hw.revision_id == E1000_REVISION_2 as u8 {
            return Err("Unsupported hardware".to_string());
        }
        Ok(())
    }
    pub fn update_mc_addr_list(&mut self, count: u32) {
        e1000_println!();
        if let Some(f) = self.hw.mac.ops.update_mc_addr_list {
            f(self, count);
        } else {
            e1000_println!("No function");
        }
    }
    pub fn mac_init_hw(&mut self) -> AdResult {
        e1000_println!();
        self.hw
            .mac
            .ops
            .init_hw
            .ok_or("No function".to_string())
            .and_then(|f| f(self))
    }
    pub fn mac_reset_hw(&mut self) -> AdResult {
        e1000_println!();
        self.hw
            .mac
            .ops
            .reset_hw
            .ok_or("No function".to_string())
            .and_then(|f| f(self))
    }
    pub fn mac_read_mac_addr(&mut self) -> AdResult {
        e1000_println!();
        self.hw
            .mac
            .ops
            .read_mac_addr
            .ok_or("No function".to_string())
            .and_then(|f| f(self))
    }
    pub fn mac_check_for_link(&mut self) -> AdResult {
        e1000_println!();
        if let Err(e) = self.hw
            .mac
            .ops
            .check_for_link
            .ok_or("No function".to_string())
            .and_then(|f| f(self))
        {
            eprintln!("(IGNORE) {:?}", e);
        }
        Ok(())
    }
    pub fn phy_read_reg(&mut self, offset: u32, data: &mut u16) -> AdResult {
        e1000_verbose_println!();
        self.hw
            .phy
            .ops
            .read_reg
            .ok_or("No function".to_string())
            .and_then(|f| f(self, offset, data))
    }
    pub fn phy_read_reg_locked(&mut self, offset: u32, data: &mut u16) -> AdResult {
        e1000_verbose_println!();
        self.hw
            .phy
            .ops
            .read_reg_locked
            .ok_or("No function".to_string())
            .and_then(|f| f(self, offset, data))
    }
    pub fn phy_write_reg(&mut self, offset: u32, data: u16) -> AdResult {
        e1000_verbose_println!();
        self.hw
            .phy
            .ops
            .write_reg
            .ok_or("No function".to_string())
            .and_then(|f| f(self, offset, data))
    }
    pub fn phy_write_reg_locked(&mut self, offset: u32, data: u16) -> AdResult {
        e1000_verbose_println!();
        self.hw
            .phy
            .ops
            .write_reg_locked
            .ok_or("No function".to_string())
            .and_then(|f| f(self, offset, data))
    }
    pub fn phy_get_info(&mut self) -> AdResult {
        e1000_println!();
        self.hw
            .phy
            .ops
            .get_info
            .ok_or("No function".to_string())
            .and_then(|f| f(self))
    }
    pub fn phy_acquire(&mut self) -> AdResult {
        e1000_println!();
        self.hw
            .phy
            .ops
            .acquire
            .ok_or("No function".to_string())
            .and_then(|f| f(self))
    }
    pub fn phy_release(&mut self) -> AdResult {
        e1000_println!();
        self.hw
            .phy
            .ops
            .release
            .ok_or("No function".to_string())
            .and_then(|f| f(self))
    }
    pub fn phy_reset(&mut self) -> AdResult {
        e1000_println!();
        self.hw
            .phy
            .ops
            .reset
            .ok_or("No function".to_string())
            .and_then(|f| f(self))
    }
    pub fn nvm_read(&mut self, offset: u16, count: u16, data: &mut [u16]) -> AdResult {
        e1000_println!();
        self.hw
            .nvm
            .ops
            .read
            .ok_or("No function".to_string())
            .and_then(|f| f(self, offset, count, data))
    }
    pub fn nvm_write(&mut self, offset: u16, count: u16, data: &mut [u16]) -> AdResult {
        e1000_println!();
        self.hw
            .nvm
            .ops
            .write
            .ok_or("No function".to_string())
            .and_then(|f| f(self, offset, count, data))
    }
    pub fn disable_promisc(&mut self) -> AdResult {
        e1000_println!();
        // u32 reg_rctl;
        // int mcnt = 0;
        let mut reg_rctl: u32;
        let mcnt: u32;

        // reg_rctl = E1000_READ_REG(&adapter->hw, E1000_RCTL);
        // reg_rctl &= (~E1000_RCTL_UPE);
        // if (if_getflags(ifp) & IFF_ALLMULTI)
        // 	mcnt = MAX_NUM_MULTICAST_ADDRESSES;
        // else
        // 	mcnt = if_multiaddr_count(ifp, MAX_NUM_MULTICAST_ADDRESSES);
        // /* Don't disable if in MAX groups */
        // if (mcnt < MAX_NUM_MULTICAST_ADDRESSES)
        // 	reg_rctl &=  (~E1000_RCTL_MPE);
        // reg_rctl &=  (~E1000_RCTL_SBP);
        // E1000_WRITE_REG(&adapter->hw, E1000_RCTL, reg_rctl);

        reg_rctl = do_read_register(self, E1000_RCTL);
        reg_rctl &= !E1000_RCTL_UPE;
        if self.ifnet.flags() & IFF_ALLMULTI != 0 {
            mcnt = MAX_NUM_MULTICAST_ADDRESSES;
        } else {
            mcnt = self.ifnet.multiaddr_count(MAX_NUM_MULTICAST_ADDRESSES);
        }
        if mcnt < MAX_NUM_MULTICAST_ADDRESSES {
            reg_rctl &= !E1000_RCTL_MPE;
        }
        reg_rctl &= !E1000_RCTL_SBP;
        do_write_register(self, E1000_RCTL, reg_rctl);
        Ok(())
    }
    pub fn set_promisc(&mut self, flags: u32) -> AdResult {
        e1000_println!();

        // struct adapter *adapter = iflib_get_softc(ctx);
        // u32 reg_rctl;
        let mut reg_rctl: u32;

        // em_disable_promisc(ctx);
        try!(self.disable_promisc());

        // reg_rctl = E1000_READ_REG(&adapter->hw, E1000_RCTL);
        reg_rctl = do_read_register(self, E1000_RCTL);

        // if (flags & IFF_PROMISC) {
        // 	reg_rctl |= (E1000_RCTL_UPE | E1000_RCTL_MPE);
        // 	/* Turn this on if you want to see bad packets */
        // 	if (em_debug_sbp)
        // 		reg_rctl |= E1000_RCTL_SBP;
        // 	E1000_WRITE_REG(&adapter->hw, E1000_RCTL, reg_rctl);
        // } else if (flags & IFF_ALLMULTI) {
        // 	reg_rctl |= E1000_RCTL_MPE;
        // 	reg_rctl &= ~E1000_RCTL_UPE;
        // 	E1000_WRITE_REG(&adapter->hw, E1000_RCTL, reg_rctl);
        // }
        // return (0);
        if flags & IFF_PROMISC != 0 {
            reg_rctl |= E1000_RCTL_UPE | E1000_RCTL_MPE;
            do_write_register(self, E1000_RCTL, reg_rctl);
        } else if flags & IFF_ALLMULTI != 0 {
            incomplete!();
        }
        Ok(())
    }
    pub fn get_wakeup(&mut self) -> AdResult {
        e1000_println!();

        // struct adapter *adapter = iflib_get_softc(ctx);
        // device_t dev = iflib_get_dev(ctx);
        // u16 eeprom_data = 0, device_id, apme_mask;

        let mut eeprom_data: [u16; 1] = [0u16];

        // adapter->has_manage = e1000_enable_mng_pass_thru(&adapter->hw);
        // apme_mask = EM_EEPROM_APME;

        self.has_manage = e1000_manage::enable_mng_pass_thru(self);
        let mut apme_mask = EM_EEPROM_APME;

        // switch (adapter->hw.mac.type) {
        // case e1000_82542:
        // case e1000_82543:
        // 	break;
        // case e1000_82544:
        // 	e1000_read_nvm(&adapter->hw,
        // 	    NVM_INIT_CONTROL2_REG, 1, &eeprom_data);
        // 	apme_mask = EM_82544_APME;
        // 	break;
        // case e1000_82546:
        // case e1000_82546_rev_3:
        // 	if (adapter->hw.bus.func == 1) {
        // 		e1000_read_nvm(&adapter->hw,
        // 		    NVM_INIT_CONTROL3_PORT_B, 1, &eeprom_data);
        // 		break;
        // 	} else
        // 		e1000_read_nvm(&adapter->hw,
        // 		    NVM_INIT_CONTROL3_PORT_A, 1, &eeprom_data);
        // 	break;
        // case e1000_82573:
        // case e1000_82583:
        // 	adapter->has_amt = TRUE;
        // 	/* FALLTHROUGH */
        // case e1000_82571:
        // case e1000_82572:
        // case e1000_80003es2lan:
        // 	if (adapter->hw.bus.func == 1) {
        // 		e1000_read_nvm(&adapter->hw,
        // 		    NVM_INIT_CONTROL3_PORT_B, 1, &eeprom_data);
        // 		break;
        // 	} else
        // 		e1000_read_nvm(&adapter->hw,
        // 		    NVM_INIT_CONTROL3_PORT_A, 1, &eeprom_data);
        // 	break;
        // case e1000_ich8lan:
        // case e1000_ich9lan:
        // case e1000_ich10lan:
        // case e1000_pchlan:
        // case e1000_pch2lan:
        // case e1000_pch_lpt:
        // case e1000_pch_spt:
        // case e1000_82575:	/* listing all igb devices */
        // case e1000_82576:
        // case e1000_82580:
        // case e1000_i350:
        // case e1000_i354:
        // case e1000_i210:
        // case e1000_i211:
        // case e1000_vfadapt:
        // case e1000_vfadapt_i350:
        // 	apme_mask = E1000_WUC_APME;
        // 	adapter->has_amt = TRUE;
        // 	eeprom_data = E1000_READ_REG(&adapter->hw, E1000_WUC);
        // 	break;
        // default:
        // 	e1000_read_nvm(&adapter->hw,
        // 	    NVM_INIT_CONTROL3_PORT_A, 1, &eeprom_data);
        // 	break;
        // }

        let mac_default = [MacType::Mac_82545];
        let mac_pch = [MacType::Mac_pch_lpt, MacType::Mac_pch_spt];

        if self.is_macs(&mac_default) {
            try!(self.nvm_read(NVM_INIT_CONTROL3_PORT_A as u16, 1, &mut eeprom_data,));
        } else if self.is_macs(&mac_pch) {
            apme_mask = E1000_WUC_APME;
            self.has_amt = true;
            eeprom_data[0] = do_read_register(self, E1000_WUC) as u16;
        } else {
            return Err("Unsupported hardware".to_string());
        }

        // if (eeprom_data & apme_mask)
        // 	adapter->wol = (E1000_WUFC_MAG | E1000_WUFC_MC);
        if eeprom_data[0] & apme_mask as u16 > 0 {
            e1000_println!("eeprom_data: {:?}, apme_mask: {:?}", eeprom_data, apme_mask);
            self.wol = E1000_WUFC_MAG | E1000_WUFC_MC;
        }
        e1000_println!("self.wol {}", self.wol);

        // Nothing more to do here for us

        /*
         * We have the eeprom settings, now apply the special cases
         * where the eeprom may be wrong or the board won't support
         * wake on lan on a particular port
         */
        // device_id = pci_get_device(dev);
        // switch (device_id) {
        // case E1000_DEV_ID_82546GB_PCIE:
        // 	adapter->wol = 0;
        // 	break;
        // case E1000_DEV_ID_82546EB_FIBER:
        // case E1000_DEV_ID_82546GB_FIBER:
        // 	/* Wake events only supported on port A for dual fiber
        // 	 * regardless of eeprom setting */
        // 	if (E1000_READ_REG(&adapter->hw, E1000_STATUS) &
        // 	    E1000_STATUS_FUNC_1)
        // 		adapter->wol = 0;
        // 	break;
        // case E1000_DEV_ID_82546GB_QUAD_COPPER_KSP3:
        // 	/* if quad port adapter, disable WoL on all but port A */
        // 	if (global_quad_port_a != 0)
        // 		adapter->wol = 0;
        // 	/* Reset for multiple quad port adapters */
        // 	if (++global_quad_port_a == 4)
        // 		global_quad_port_a = 0;
        // 	break;
        // case E1000_DEV_ID_82571EB_FIBER:
        // 	/* Wake events only supported on port A for dual fiber
        // 	 * regardless of eeprom setting */
        // 	if (E1000_READ_REG(&adapter->hw, E1000_STATUS) &
        // 	    E1000_STATUS_FUNC_1)
        // 		adapter->wol = 0;
        // 	break;
        // case E1000_DEV_ID_82571EB_QUAD_COPPER:
        // case E1000_DEV_ID_82571EB_QUAD_FIBER:
        // case E1000_DEV_ID_82571EB_QUAD_COPPER_LP:
        // 	/* if quad port adapter, disable WoL on all but port A */
        // 	if (global_quad_port_a != 0)
        // 		adapter->wol = 0;
        // 	/* Reset for multiple quad port adapters */
        // 	if (++global_quad_port_a == 4)
        // 		global_quad_port_a = 0;
        // 	break;
        // }
        let dev_id: u32 = self.dev.pci_get_device();
        match dev_id {
            E1000_DEV_ID_82546GB_PCIE => panic!("Unsupported hardware"),
            E1000_DEV_ID_82546EB_FIBER => panic!("Unsupported hardware"),
            E1000_DEV_ID_82546GB_FIBER => panic!("Unsupported hardware"),
            E1000_DEV_ID_82546GB_QUAD_COPPER_KSP3 => panic!("Unsupported hardware"),
            E1000_DEV_ID_82571EB_FIBER => panic!("Unsupported hardware"),
            E1000_DEV_ID_82571EB_QUAD_COPPER => panic!("Unsupported hardware"),
            E1000_DEV_ID_82571EB_QUAD_FIBER => panic!("Unsupported hardware"),
            E1000_DEV_ID_82571EB_QUAD_COPPER_LP => panic!("Unsupported hardware"),
            _ => (),
        }

        Ok(())
    }
    pub fn setup_interface(&mut self) -> AdResult {
        e1000_println!();

        // struct ifnet *ifp = iflib_get_ifp(ctx);
        // let ifp: *mut ifnet = iflib_sys::iflib_get_ifp()
        // struct adapter *adapter = iflib_get_softc(ctx);
        // if_softc_ctx_t scctx = adapter->shared;
        // uint64_t cap = 0;

        // INIT_DEBUGOUT("em_setup_interface: begin");

        /* TSO parameters */
        // if_sethwtsomax(ifp, IP_MAXPACKET);
        self.ifnet.set_hwtsomax(IP_MAXPACKET);

        /* Take m_pullup(9)'s in em_xmit() w/ TSO into acount. */
        // if_sethwtsomaxsegcount(ifp, EM_MAX_SCATTER - 5);
        // if_sethwtsomaxsegsize(ifp, EM_TSO_SEG_SIZE);
        self.ifnet.set_hwtsomax_segcount(EM_MAX_SCATTER - 5);
        self.ifnet.set_hwtsomax_segsize(EM_TSO_SEG_SIZE);

        /* Single Queue */
        // if (adapter->tx_num_queues == 1) {
        // 	if_setsendqlen(ifp, scctx->isc_ntxd[0] - 1);
        // 	if_setsendqready(ifp);
        // }
        if self.iflib_shared.isc_ntxqsets == 1 {
            self.ifnet.set_sendqlen(self.iflib_shared.isc_ntxd[0] - 1);
            self.ifnet.set_sendqready();
        }
        let mut cap: u64;

        // cap = IFCAP_HWCSUM | IFCAP_VLAN_HWCSUM | IFCAP_TSO4;
        // cap |= IFCAP_VLAN_HWTAGGING | IFCAP_VLAN_HWTSO | IFCAP_VLAN_MTU;
        cap = (IFCAP_HWCSUM | IFCAP_VLAN_HWCSUM | IFCAP_TSO4) as u64;
        cap |= (IFCAP_VLAN_HWTAGGING | IFCAP_VLAN_HWTSO | IFCAP_VLAN_MTU) as u64;

        /*
         * Tell the upper layer(s) we
         * support full VLAN capability
         */
        // if_setifheaderlen(ifp, sizeof(struct ether_vlan_header));
        // if_setcapabilitiesbit(ifp, cap, 0);
        self.ifnet
            .set_ifheaderlen(kernel::mem::size_of::<ether_vlan_header>() as i32);
        self.ifnet.set_capabilitiesbit(cap as i32, 0);

        /*
         * Don't turn this on by default, if vlans are
         * created on another pseudo device (eg. lagg)
         * then vlan events are not passed thru, breaking
         * operation, but with HW FILTER off it works. If
         * using vlans directly on the em driver you can
         * enable this and get full hardware tag filtering.
         */
        // if_setcapabilitiesbit(ifp, IFCAP_VLAN_HWFILTER,0);
        self.ifnet
            .set_capabilitiesbit(IFCAP_VLAN_HWFILTER as i32, 0);

        /* Enable only WOL MAGIC by default */
        // if (adapter->wol) {
        // 	if_setcapenablebit(ifp, IFCAP_WOL_MAGIC,
        // 		    IFCAP_WOL_MCAST| IFCAP_WOL_UCAST);
        // } else {
        // 	if_setcapenablebit(ifp, 0, IFCAP_WOL_MAGIC |
        // 		     IFCAP_WOL_MCAST| IFCAP_WOL_UCAST);
        // }
        e1000_println!("self.wol {}", self.wol);
        if self.wol > 0 {
            e1000_println!("setting WOL");
            self.ifnet.set_capenablebit(
                IFCAP_WOL_MAGIC as i32,
                (IFCAP_WOL_MCAST | IFCAP_WOL_UCAST) as i32,
            );
        } else {
            e1000_println!("clearing WOL");
            self.ifnet.set_capenablebit(
                0,
                (IFCAP_WOL_MAGIC | IFCAP_WOL_MCAST | IFCAP_WOL_UCAST) as i32,
            );
        }

        /*
         * Specify the media types supported by this adapter and register
         * callbacks to update media and link information
         */
        // if ((adapter->hw.phy.media_type == e1000_media_type_fiber) ||
        //     (adapter->hw.phy.media_type == e1000_media_type_internal_serdes)) {
        // 	u_char fiber_type = IFM_1000_SX;	/* default type */

        // 	if (adapter->hw.mac.type == e1000_82545)
        // 		fiber_type = IFM_1000_LX;
        // 	ifmedia_add(adapter->media, IFM_ETHER | fiber_type | IFM_FDX, 0, NULL);
        // 	ifmedia_add(adapter->media, IFM_ETHER | fiber_type, 0, NULL);
        // } else {

        // 	ifmedia_add(adapter->media, IFM_ETHER | IFM_10_T, 0, NULL);
        // 	ifmedia_add(adapter->media, IFM_ETHER | IFM_10_T | IFM_FDX, 0, NULL);
        // 	ifmedia_add(adapter->media, IFM_ETHER | IFM_100_TX, 0, NULL);
        // 	ifmedia_add(adapter->media, IFM_ETHER | IFM_100_TX | IFM_FDX, 0, NULL);
        if !self.is_copper() {
            return Err("Unsupported hardware".to_string());
        }

        self.ifmedia
            .add((IFM_ETHER | IFM_10_T) as i32, 0, kernel::ptr::null_mut());
        self.ifmedia.add(
            (IFM_ETHER | IFM_10_T | IFM_FDX) as i32,
            0,
            kernel::ptr::null_mut(),
        );
        self.ifmedia
            .add((IFM_ETHER | IFM_100_TX) as i32, 0, kernel::ptr::null_mut());
        self.ifmedia.add(
            (IFM_ETHER | IFM_100_TX | IFM_FDX) as i32,
            0,
            kernel::ptr::null_mut(),
        );

        // 	if (adapter->hw.phy.type != e1000_phy_ife) {
        // 		ifmedia_add(adapter->media, IFM_ETHER | IFM_1000_T | IFM_FDX, 0, NULL);
        // 		ifmedia_add(adapter->media, IFM_ETHER | IFM_1000_T, 0, NULL);
        // 	}

        if self.hw.phy.phy_type != PhyType::Type_ife {
            self.ifmedia.add(
                (IFM_ETHER | IFM_1000_T | IFM_FDX) as i32,
                0,
                kernel::ptr::null_mut(),
            );
            self.ifmedia
                .add((IFM_ETHER | IFM_1000_T) as i32, 0, kernel::ptr::null_mut());
        }
        // }
        // ifmedia_add(adapter->media, IFM_ETHER | IFM_AUTO, 0, NULL);
        // ifmedia_set(adapter->media, IFM_ETHER | IFM_AUTO);
        // return (0);

        self.ifmedia
            .add((IFM_ETHER | IFM_AUTO) as i32, 0, kernel::ptr::null_mut());
        self.ifmedia.set((IFM_ETHER | IFM_AUTO) as i32);

        Ok(())
    }
    pub fn setup_vlan_hw_support(&mut self) {
        e1000_println!();
        // struct e1000_hw *hw = &adapter->hw;
        // u32 reg;

        /*
         * We get here thru init_locked, meaning
         * a soft reset, this has already cleared
         * the VFTA and other state, so if there
         * have been no vlan's registered do nothing.
         */
        // if (adapter->num_vlans == 0)
        // 	return;
        if self.num_vlans == 0 {
            e1000_println!("No vlans, returning early");
            return;
        }

        // /*
        //  * A soft reset zero's out the VFTA, so
        //  * we need to repopulate it now.
        //  */
        // for (int i = 0; i < EM_VFTA_SIZE; i++)
        // 	if (adapter->shadow_vfta[i] != 0)
        // 		E1000_WRITE_REG_ARRAY(hw, E1000_VFTA,
        // 		    i, adapter->shadow_vfta[i]);

        // reg = E1000_READ_REG(hw, E1000_CTRL);
        // reg |= E1000_CTRL_VME;
        // E1000_WRITE_REG(hw, E1000_CTRL, reg);

        // /* Enable the Filter Table */
        // reg = E1000_READ_REG(hw, E1000_RCTL);
        // reg &= ~E1000_RCTL_CFIEN;
        // reg |= E1000_RCTL_VFE;
        // E1000_WRITE_REG(hw, E1000_RCTL, reg);
        incomplete!();
    }
    pub fn reset(&mut self) -> AdResult {
        e1000_println!();

        // device_t dev = iflib_get_dev(ctx);
        // struct adapter *adapter = iflib_get_softc(ctx);
        // struct ifnet *ifp = iflib_get_ifp(ctx);
        // struct e1000_hw *hw = &adapter->hw;
        // u16 rx_buffer_size;
        // u32 pba;

        // INIT_DEBUGOUT("em_reset: begin");

        /* Let the firmware know the OS is in control */
        // em_get_hw_control(adapter);
        self.get_hw_control();

        /* Set up smart power down as default off on newer adapters. */
        // if (!em_smart_pwr_down && (hw->mac.type == e1000_82571 ||
        //     hw->mac.type == e1000_82572)) {
        // 	u16 phy_tmp = 0;

        // 	/* Speed up time to link by disabling smart power down. */
        // 	e1000_read_phy_reg(hw, IGP02E1000_PHY_POWER_MGMT, &phy_tmp);
        // 	phy_tmp &= ~IGP02E1000_PM_SPD;
        // 	e1000_write_phy_reg(hw, IGP02E1000_PHY_POWER_MGMT, phy_tmp);
        // }
        if self.is_macs(&[MacType::Mac_82571, MacType::Mac_82572]) {
            return Err("Unsupported hardware".to_string());
        }

        /*
         * Packet Buffer Allocation (PBA)
         * Writing PBA sets the receive portion of the buffer
         * the remainder is used for the transmit buffer.
         */
        // switch (hw->mac.type) {
        // /* Total Packet Buffer on these is 48K */
        // case e1000_82571:
        // case e1000_82572:
        // case e1000_80003es2lan:
        // 		pba = E1000_PBA_32K; /* 32K for Rx, 16K for Tx */
        // 	break;
        // case e1000_82573: /* 82573: Total Packet Buffer is 32K */
        // 		pba = E1000_PBA_12K; /* 12K for Rx, 20K for Tx */
        // 	break;
        // case e1000_82574:
        // case e1000_82583:
        // 		pba = E1000_PBA_20K; /* 20K for Rx, 20K for Tx */
        // 	break;
        // case e1000_ich8lan:
        // 	pba = E1000_PBA_8K;
        // 	break;
        // case e1000_ich9lan:
        // case e1000_ich10lan:
        // 	/* Boost Receive side for jumbo frames */
        // 	if (adapter->hw.mac.max_frame_size > 4096)
        // 		pba = E1000_PBA_14K;
        // 	else
        // 		pba = E1000_PBA_10K;
        // 	break;
        // case e1000_pchlan:
        // case e1000_pch2lan:
        // case e1000_pch_lpt:
        // case e1000_pch_spt:
        // case e1000_pch_cnp:
        // 	pba = E1000_PBA_26K;
        // 	break;
        // case e1000_82575:
        // 	pba = E1000_PBA_32K;
        // 	break;
        // case e1000_82576:
        // case e1000_vfadapt:
        // 	pba = E1000_READ_REG(hw, E1000_RXPBS);
        // 	pba &= E1000_RXPBS_SIZE_MASK_82576;
        // 	break;
        // case e1000_82580:
        // case e1000_i350:
        // case e1000_i354:
        // case e1000_vfadapt_i350:
        // 	pba = E1000_READ_REG(hw, E1000_RXPBS);
        // 	pba = e1000_rxpbs_adjust_82580(pba);
        // 	break;
        // case e1000_i210:
        // case e1000_i211:
        // 	pba = E1000_PBA_34K;
        // 	break;
        // default:
        // 	if (adapter->hw.mac.max_frame_size > 8192)
        // 		pba = E1000_PBA_40K; /* 40K for Rx, 24K for Tx */
        // 	else
        // 		pba = E1000_PBA_48K; /* 48K for Rx, 16K for Tx */
        // }
        let pba;
        if self.is_mac(MacType::Mac_82545) {
            if self.hw.mac.max_frame_size > 8192 {
                pba = E1000_PBA_40K;
            } else {
                pba = E1000_PBA_48K;
            }
        } else if self.is_macs(&[MacType::Mac_pch_lpt, MacType::Mac_pch_spt]) {
            pba = E1000_PBA_26K;
        } else {
            return Err("Unsupported hardware".to_string());
        }

        // /* Special needs in case of Jumbo frames */
        // if ((hw->mac.type == e1000_82575) && (ifp->if_mtu > ETHERMTU)) {
        // 	u32 tx_space, min_tx, min_rx;
        // 	pba = E1000_READ_REG(hw, E1000_PBA);
        // 	tx_space = pba >> 16;
        // 	pba &= 0xffff;
        // 	min_tx = (adapter->hw.mac.max_frame_size +
        // 	    sizeof(struct e1000_tx_desc) - ETHERNET_FCS_SIZE) * 2;
        // 	min_tx = roundup2(min_tx, 1024);
        // 	min_tx >>= 10;
        // 	min_rx = adapter->hw.mac.max_frame_size;
        // 	min_rx = roundup2(min_rx, 1024);
        // 	min_rx >>= 10;
        // 	if (tx_space < min_tx &&
        // 	    ((min_tx - tx_space) < pba)) {
        // 		pba = pba - (min_tx - tx_space);
        // 		/*
        // 		 * if short on rx space, rx wins
        // 		 * and must trump tx adjustment
        // 		 */
        // 		if (pba < min_rx)
        // 			pba = min_rx;
        // 	}
        // 	E1000_WRITE_REG(hw, E1000_PBA, pba);
        // }
        if self.is_mac(MacType::Mac_82575) && self.ifnet.mtu() > ETHERMTU as i32 {
            return Err("Unsupported hardware".to_string());
        }

        // if (hw->mac.type < igb_mac_min)
        // 	E1000_WRITE_REG(&adapter->hw, E1000_PBA, pba);
        if self.hw.mac.mac_type < MacType::IGB_MAC_MIN {
            do_write_register(self, E1000_PBA, pba);
        }
        // INIT_DEBUGOUT1("em_reset: pba=%dK",pba);
        e1000_println!("pba = {}K", pba);

        /*
         * These parameters control the automatic generation (Tx) and
         * response (Rx) to Ethernet PAUSE frames.
         * - High water mark should allow for at least two frames to be
         *   received after sending an XOFF.
         * - Low water mark works best when it is very near the high water mark.
         *   This allows the receiver to restart by sending XON when it has
         *   drained a bit. Here we use an arbitrary value of 1500 which will
         *   restart after one full frame is pulled from the buffer. There
         *   could be several smaller frames in the buffer and if so they will
         *   not trigger the XON until their total number reduces the buffer
         *   by 1500.
         * - The pause time is fairly large at 1000 x 512ns = 512 usec.
         */
        // rx_buffer_size = (pba & 0xffff) << 10;
        // hw->fc.high_water = rx_buffer_size -
        //     roundup2(adapter->hw.mac.max_frame_size, 1024);
        // hw->fc.low_water = hw->fc.high_water - 1500;

        let rx_buffer_size = (pba & 0xffff) << 10;
        self.hw.fc.high_water = rx_buffer_size - roundup2!(self.hw.mac.max_frame_size, 1024);
        self.hw.fc.low_water = self.hw.fc.high_water - 1500;

        // if (adapter->fc) /* locally set flow control value? */
        // 	hw->fc.requested_mode = adapter->fc;
        // else
        // 	hw->fc.requested_mode = e1000_fc_full;
        if self.fc != FcMode::None {
            self.hw.fc.requested_mode = self.fc;
        } else {
            self.hw.fc.requested_mode = FcMode::Full;
        }

        // if (hw->mac.type == e1000_80003es2lan)
        // 	hw->fc.pause_time = 0xFFFF;
        // else
        // 	hw->fc.pause_time = EM_FC_PAUSE_TIME;
        if self.is_mac(MacType::Mac_80003es2lan) {
            self.hw.fc.pause_time = 0xFFFF;
        } else {
            self.hw.fc.pause_time = EM_FC_PAUSE_TIME as u16;
        }

        // hw->fc.send_xon = TRUE;
        self.hw.fc.send_xon = true;

        /* Device specific overrides/settings */
        // switch (hw->mac.type) {
        // case e1000_pchlan:
        // 	/* Workaround: no TX flow ctrl for PCH */
        // 	hw->fc.requested_mode = e1000_fc_rx_pause;
        // 	hw->fc.pause_time = 0xFFFF; /* override */
        // 	if (if_getmtu(ifp) > ETHERMTU) {
        // 		hw->fc.high_water = 0x3500;
        // 		hw->fc.low_water = 0x1500;
        // 	} else {
        // 		hw->fc.high_water = 0x5000;
        // 		hw->fc.low_water = 0x3000;
        // 	}
        // 	hw->fc.refresh_time = 0x1000;
        // 	break;
        // case e1000_pch2lan:
        // case e1000_pch_lpt:
        // case e1000_pch_spt:
        // case e1000_pch_cnp:
        // 	hw->fc.high_water = 0x5C20;
        // 	hw->fc.low_water = 0x5048;
        // 	hw->fc.pause_time = 0x0650;
        // 	hw->fc.refresh_time = 0x0400;
        // 	/* Jumbos need adjusted PBA */
        // 	if (if_getmtu(ifp) > ETHERMTU)
        // 		E1000_WRITE_REG(hw, E1000_PBA, 12);
        // 	else
        // 		E1000_WRITE_REG(hw, E1000_PBA, 26);
        // 	break;
        // case e1000_82575:
        // case e1000_82576:
        // 	/* 8-byte granularity */
        // 	hw->fc.low_water = hw->fc.high_water - 8;
        // 	break;
        // case e1000_82580:
        // case e1000_i350:
        // case e1000_i354:
        // case e1000_i210:
        // case e1000_i211:
        // case e1000_vfadapt:
        // case e1000_vfadapt_i350:
        // 	/* 16-byte granularity */
        // 	hw->fc.low_water = hw->fc.high_water - 16;
        // 	break;
        // case e1000_ich9lan:
        // case e1000_ich10lan:
        // 	if (if_getmtu(ifp) > ETHERMTU) {
        // 		hw->fc.high_water = 0x2800;
        // 		hw->fc.low_water = hw->fc.high_water - 8;
        // 		break;
        // 	}
        // 	/* FALLTHROUGH */
        // default:
        // 	if (hw->mac.type == e1000_80003es2lan)
        // 		hw->fc.pause_time = 0xFFFF;
        // 	break;
        // }
        if self.is_mac(MacType::Mac_82545) {
            // NOOP
        } else if self.is_macs(&[MacType::Mac_pch_lpt, MacType::Mac_pch_spt]) {
            // 	hw->fc.high_water = 0x5C20;
            // 	hw->fc.low_water = 0x5048;
            // 	hw->fc.pause_time = 0x0650;
            // 	hw->fc.refresh_time = 0x0400;
            // 	/* Jumbos need adjusted PBA */
            // 	if (if_getmtu(ifp) > ETHERMTU)
            // 		E1000_WRITE_REG(hw, E1000_PBA, 12);
            // 	else
            // 		E1000_WRITE_REG(hw, E1000_PBA, 26);
            // 	break;
            self.hw.fc.high_water = 0x5c20;
            self.hw.fc.low_water = 0x5048;
            self.hw.fc.pause_time = 0x0650;
            self.hw.fc.refresh_time = 0x0400;
            if self.ifnet.mtu() > ETHERMTU as i32 {
                do_write_register(self, E1000_PBA, 12);
            } else {
                do_write_register(self, E1000_PBA, 26);
            }
        } else {
            return Err("Unsupported hardware".to_string());
        }

        /* Issue a global reset */
        // e1000_reset_hw(hw);
        try!(self.mac_reset_hw());

        // if (adapter->hw.mac.type >= igb_mac_min) {
        // 	E1000_WRITE_REG(hw, E1000_WUC, 0);
        // } else {
        // 	E1000_WRITE_REG(hw, E1000_WUFC, 0);
        // 	em_disable_aspm(adapter);
        // }
        if self.hw.mac.mac_type >= MacType::IGB_MAC_MIN {
            do_write_register(self, E1000_WUC, 0);
        } else {
            do_write_register(self, E1000_WUFC, 0);
            self.disable_aspm();
        }

        // if (adapter->flags & IGB_MEDIA_RESET) {
        // 	e1000_setup_init_funcs(hw, TRUE);
        // 	e1000_get_bus_info(hw);
        // 	adapter->flags &= ~IGB_MEDIA_RESET;
        // }
        // e1000_println!("Do we use adapter.flags?");

        /* and a re-init */
        // if (e1000_init_hw(hw) < 0) {
        // 	device_printf(dev, "Hardware Initialization Failed\n");
        // 	return;
        // }
        try!(self.mac_init_hw());

        // if (adapter->hw.mac.type >= igb_mac_min)
        // 	igb_init_dmac(adapter, pba);
        if self.hw.mac.mac_type >= MacType::IGB_MAC_MIN {
            return Err("Unsupported hardware".to_string());
        }

        // E1000_WRITE_REG(hw, E1000_VET, ETHERTYPE_VLAN);
        // e1000_get_phy_info(hw);
        // e1000_check_for_link(hw);
        do_write_register(self, E1000_VET, ETHERTYPE_VLAN);
        if let Err(e) = self.phy_get_info() {
            e1000_println!("(IGNORING) {:?}", e);
        }
        if let Err(e) = self.mac_check_for_link() {
            e1000_println!("(IGNORING) {:?}", e);
        }
        Ok(())
    }
    pub fn lem_smartspeed(&mut self) -> AdResult {
        e1000_println!();
        // u16 phy_tmp;
        let mut phy_tmp: u16 = 0;

        // if (adapter->link_active || (adapter->hw.phy.type != e1000_phy_igp) ||
        //     adapter->hw.mac.autoneg == 0 ||
        //     (adapter->hw.phy.autoneg_advertised & ADVERTISE_1000_FULL) == 0)
        // 	return;
        if self.link_active || self.hw.phy.phy_type != PhyType::Type_igp
            || self.hw.mac.autoneg == false
            || self.hw.phy.autoneg_advertised & ADVERTISE_1000_FULL == 0
        {
            return Ok(());
        }

        // if (adapter->smartspeed == 0) {
        // 	/* If Master/Slave config fault is asserted twice,
        // 	 * we assume back-to-back */
        // 	e1000_read_phy_reg(&adapter->hw, PHY_1000T_STATUS, &phy_tmp);
        // 	if (!(phy_tmp & SR_1000T_MS_CONFIG_FAULT))
        // 		return;
        // 	e1000_read_phy_reg(&adapter->hw, PHY_1000T_STATUS, &phy_tmp);
        // 	if (phy_tmp & SR_1000T_MS_CONFIG_FAULT) {
        // 		e1000_read_phy_reg(&adapter->hw,
        // 		    PHY_1000T_CTRL, &phy_tmp);
        // 		if(phy_tmp & CR_1000T_MS_ENABLE) {
        // 			phy_tmp &= ~CR_1000T_MS_ENABLE;
        // 			e1000_write_phy_reg(&adapter->hw,
        // 			    PHY_1000T_CTRL, phy_tmp);
        // 			adapter->smartspeed++;
        // 			if(adapter->hw.mac.autoneg &&
        // 			   !e1000_copper_link_autoneg(&adapter->hw) &&
        // 			   !e1000_read_phy_reg(&adapter->hw,
        // 			    PHY_CONTROL, &phy_tmp)) {
        // 				phy_tmp |= (MII_CR_AUTO_NEG_EN |
        // 					    MII_CR_RESTART_AUTO_NEG);
        // 				e1000_write_phy_reg(&adapter->hw,
        // 				    PHY_CONTROL, phy_tmp);
        // 			}
        // 		}
        // 	}
        // 	return;
        if self.smartspeed == 0 {
            try!(self.phy_read_reg(PHY_1000T_STATUS, &mut phy_tmp));
            if phy_tmp & SR_1000T_MS_CONFIG_FAULT == 0 {
                return Ok(());
            }
            try!(self.phy_read_reg(PHY_1000T_STATUS, &mut phy_tmp));
            if phy_tmp & SR_1000T_MS_CONFIG_FAULT != 0 {
                try!(self.phy_read_reg(PHY_1000T_CTRL, &mut phy_tmp));
                if phy_tmp & CR_1000T_MS_ENABLE != 0 {
                    phy_tmp &= !CR_1000T_MS_ENABLE;
                    try!(self.phy_write_reg(PHY_1000T_CTRL, phy_tmp));
                    self.smartspeed += 1;
                    try!(e1000_phy::copper_link_autoneg(self));
                    if self.hw.mac.autoneg {
                        try!(self.phy_read_reg(PHY_CONTROL, &mut phy_tmp));
                        phy_tmp |= MII_CR_AUTO_NEG_EN | MII_CR_RESTART_AUTO_NEG;
                        try!(self.phy_write_reg(PHY_CONTROL, phy_tmp));
                    }
                }
            }
        }
        // } else if(adapter->smartspeed == EM_SMARTSPEED_DOWNSHIFT) {
        // 	/* If still no link, perhaps using 2/3 pair cable */
        // 	e1000_read_phy_reg(&adapter->hw, PHY_1000T_CTRL, &phy_tmp);
        // 	phy_tmp |= CR_1000T_MS_ENABLE;
        // 	e1000_write_phy_reg(&adapter->hw, PHY_1000T_CTRL, phy_tmp);
        // 	if(adapter->hw.mac.autoneg &&
        // 	   !e1000_copper_link_autoneg(&adapter->hw) &&
        // 	   !e1000_read_phy_reg(&adapter->hw, PHY_CONTROL, &phy_tmp)) {
        // 		phy_tmp |= (MII_CR_AUTO_NEG_EN |
        // 			    MII_CR_RESTART_AUTO_NEG);
        // 		e1000_write_phy_reg(&adapter->hw, PHY_CONTROL, phy_tmp);
        // 	}
        // }
        else if self.smartspeed == EM_SMARTSPEED_DOWNSHIFT {
            try!(self.phy_read_reg(PHY_1000T_CTRL, &mut phy_tmp));
            phy_tmp |= CR_1000T_MS_ENABLE;
            try!(self.phy_write_reg(PHY_1000T_CTRL, phy_tmp));
            try!(e1000_phy::copper_link_autoneg(self));
            if self.hw.mac.autoneg {
                try!(self.phy_read_reg(PHY_CONTROL, &mut phy_tmp));
                phy_tmp |= MII_CR_AUTO_NEG_EN | MII_CR_RESTART_AUTO_NEG;
                try!(self.phy_write_reg(PHY_CONTROL, phy_tmp));
            }
        }
        // /* Restart process after EM_SMARTSPEED_MAX iterations */
        // if(adapter->smartspeed++ == EM_SMARTSPEED_MAX)
        // 	adapter->smartspeed = 0;
        self.smartspeed += 1;
        if self.smartspeed == EM_SMARTSPEED_MAX {
            self.smartspeed = 0;
        }
        Ok(())
    }
    pub fn disable_aspm(&mut self) {
        e1000_println!();

        // int base, reg;
        // u16 link_cap,link_ctrl;
        // device_t dev = adapter->dev;

        // switch (adapter->hw.mac.type) {
        // case e1000_82573:
        // case e1000_82574:
        // case e1000_82583:
        // 	break;
        // default:
        // 	return;
        // }
        let macs = [MacType::Mac_82573, MacType::Mac_82574, MacType::Mac_82583];
        if self.is_not_macs(&macs) {
            return;
        }
        panic!("Unsupported hardware");

        // if (pci_find_cap(dev, PCIY_EXPRESS, &base) != 0)
        // 	return;
        // reg = base + PCIER_LINK_CAP;
        // link_cap = pci_read_config(dev, reg, 2);
        // if ((link_cap & PCIEM_LINK_CAP_ASPM) == 0)
        // 	return;
        // reg = base + PCIER_LINK_CTL;
        // link_ctrl = pci_read_config(dev, reg, 2);
        // link_ctrl &= ~PCIEM_LINK_CTL_ASPMC;
        // pci_write_config(dev, reg, link_ctrl, 2);
    }
    pub fn update_stats_counters(&mut self) {
        e1000_println!();

        if self.hw.phy.media_type == MediaType::Copper
            || do_read_register(self, E1000_STATUS) & E1000_STATUS_LU > 0
        {
            self.stats.symerrs += do_read_register(self, E1000_SYMERRS) as u64;
            self.stats.sec += do_read_register(self, E1000_SEC) as u64;
        }

        self.stats.crcerrs += do_read_register(self, E1000_CRCERRS) as u64;
        self.stats.mpc += do_read_register(self, E1000_MPC) as u64;
        self.stats.scc += do_read_register(self, E1000_SCC) as u64;
        self.stats.ecol += do_read_register(self, E1000_ECOL) as u64;

        self.stats.mcc += do_read_register(self, E1000_MCC) as u64;
        self.stats.latecol += do_read_register(self, E1000_LATECOL) as u64;
        self.stats.colc += do_read_register(self, E1000_COLC) as u64;
        self.stats.dc += do_read_register(self, E1000_DC) as u64;
        self.stats.rlec += do_read_register(self, E1000_RLEC) as u64;
        self.stats.xonrxc += do_read_register(self, E1000_XONRXC) as u64;
        self.stats.xontxc += do_read_register(self, E1000_XONTXC) as u64;
        self.stats.xoffrxc += do_read_register(self, E1000_XOFFRXC) as u64;
        /*
         ** For watchdog management we need to know if we have been
         ** paused during the last interval, so capture that here.
         */
        self.iflib_shared.isc_pause_frames = self.stats.xoffrxc as u32;
        self.stats.xofftxc += do_read_register(self, E1000_XOFFTXC) as u64;
        self.stats.fcruc += do_read_register(self, E1000_FCRUC) as u64;
        self.stats.prc64 += do_read_register(self, E1000_PRC64) as u64;
        self.stats.prc127 += do_read_register(self, E1000_PRC127) as u64;
        self.stats.prc255 += do_read_register(self, E1000_PRC255) as u64;
        self.stats.prc511 += do_read_register(self, E1000_PRC511) as u64;
        self.stats.prc1023 += do_read_register(self, E1000_PRC1023) as u64;
        self.stats.prc1522 += do_read_register(self, E1000_PRC1522) as u64;
        self.stats.gprc += do_read_register(self, E1000_GPRC) as u64;
        self.stats.bprc += do_read_register(self, E1000_BPRC) as u64;
        self.stats.mprc += do_read_register(self, E1000_MPRC) as u64;
        self.stats.gptc += do_read_register(self, E1000_GPTC) as u64;

        /* For the 64-bit byte counters the low dword must be read first. */
        /* Both registers clear on the read of the high dword */

        self.stats.gorc += do_read_register(self, E1000_GORCL) as u64
            + ((do_read_register(self, E1000_GORCH) as u64) << 32);
        self.stats.gotc += do_read_register(self, E1000_GOTCL) as u64
            + ((do_read_register(self, E1000_GOTCH) as u64) << 32);

        self.stats.rnbc += do_read_register(self, E1000_RNBC) as u64;
        self.stats.ruc += do_read_register(self, E1000_RUC) as u64;
        self.stats.rfc += do_read_register(self, E1000_RFC) as u64;
        self.stats.roc += do_read_register(self, E1000_ROC) as u64;
        self.stats.rjc += do_read_register(self, E1000_RJC) as u64;

        self.stats.tor += do_read_register(self, E1000_TORH) as u64;
        self.stats.tot += do_read_register(self, E1000_TOTH) as u64;

        self.stats.tpr += do_read_register(self, E1000_TPR) as u64;
        self.stats.tpt += do_read_register(self, E1000_TPT) as u64;
        self.stats.ptc64 += do_read_register(self, E1000_PTC64) as u64;
        self.stats.ptc127 += do_read_register(self, E1000_PTC127) as u64;
        self.stats.ptc255 += do_read_register(self, E1000_PTC255) as u64;
        self.stats.ptc511 += do_read_register(self, E1000_PTC511) as u64;
        self.stats.ptc1023 += do_read_register(self, E1000_PTC1023) as u64;
        self.stats.ptc1522 += do_read_register(self, E1000_PTC1522) as u64;
        self.stats.mptc += do_read_register(self, E1000_MPTC) as u64;
        self.stats.bptc += do_read_register(self, E1000_BPTC) as u64;

        /* Interrupt Counts */

        self.stats.iac += do_read_register(self, E1000_IAC) as u64;
        self.stats.icrxptc += do_read_register(self, E1000_ICRXPTC) as u64;
        self.stats.icrxatc += do_read_register(self, E1000_ICRXATC) as u64;
        self.stats.ictxptc += do_read_register(self, E1000_ICTXPTC) as u64;
        self.stats.ictxatc += do_read_register(self, E1000_ICTXATC) as u64;
        self.stats.ictxqec += do_read_register(self, E1000_ICTXQEC) as u64;
        self.stats.ictxqmtc += do_read_register(self, E1000_ICTXQMTC) as u64;
        self.stats.icrxdmtc += do_read_register(self, E1000_ICRXDMTC) as u64;
        self.stats.icrxoc += do_read_register(self, E1000_ICRXOC) as u64;

        if self.hw.mac.mac_type >= MacType::Mac_82543 {
            self.stats.algnerrc += do_read_register(self, E1000_ALGNERRC) as u64;
            self.stats.rxerrc += do_read_register(self, E1000_RXERRC) as u64;
            self.stats.tncrs += do_read_register(self, E1000_TNCRS) as u64;
            self.stats.cexterr += do_read_register(self, E1000_CEXTERR) as u64;
            self.stats.tsctc += do_read_register(self, E1000_TSCTC) as u64;
            self.stats.tsctfc += do_read_register(self, E1000_TSCTFC) as u64;
        }
    }
    pub fn update_admin_status(&mut self) -> AdResult {
        e1000_println!();

        // struct adapter *adapter = iflib_get_softc(ctx);
        // static	struct e1000_hw *hw = &adapter->hw;
        // struct ifnet *ifp = iflib_get_ifp(ctx);
        // device_t dev = iflib_get_dev(ctx);
        // u32 link_check, thstat, ctrl;
        // link_check = thstat = ctrl = 0;
        let mut link_check: bool = false;
        let mut thstat: u32 = 0;
        let mut ctrl: u32 = 0;

        /* Get the cached link value or read phy for real */
        // switch (hw->phy.media_type) {
        // case e1000_media_type_copper:
        // 	if (hw->mac.get_link_status) {
        // 		if (hw->mac.type == e1000_pch_spt)
        // 			msec_delay(50);
        // 		/* Do the work to read phy */
        // 		e1000_check_for_link(hw);
        // 		link_check = !hw->mac.get_link_status;
        // 		if (link_check) /* ESB2 fix */
        // 			e1000_cfg_on_link_up(hw);
        // 	} else {
        // 		link_check = TRUE;
        // 	}
        // 	break;
        e1000_println!(
            "self.hw.mac.get_link_status is {:?}",
            self.hw.mac.get_link_status
        );

        match self.hw.phy.media_type {
            MediaType::Copper => {
                if self.hw.mac.get_link_status {
                    if self.is_mac(MacType::Mac_pch_spt) {
                        do_msec_delay(50);
                    }
                    /* Do the work to read phy */
                    if let Err(e) = self.mac_check_for_link() {
                        e1000_println!("(IGNORING) {:?}", e);
                    }
                    link_check = !self.hw.mac.get_link_status;
                    if link_check {
                        try!(self.cfg_on_link_up());
                    }
                } else {
                    link_check = true;
                }
            }
            // case e1000_media_type_fiber:
            // 	e1000_check_for_link(hw);
            // 	link_check = (E1000_READ_REG(hw, E1000_STATUS) &
            // 		    E1000_STATUS_LU);
            // 	break;
            // case e1000_media_type_internal_serdes:
            // 	e1000_check_for_link(hw);
            // 	link_check = adapter->hw.mac.serdes_has_link;
            // 	break;
            // /* VF device is type_unknown */
            // case e1000_media_type_unknown:
            // 	e1000_check_for_link(hw);
            // 	link_check = !hw->mac.get_link_status;
            // 	/* FALLTHROUGH */
            // default:
            // 	break;
            // }
            MediaType::Unknown => {
                e1000_println!("MediaType unknown!");
                if let Err(e) = self.mac_check_for_link() {
                    e1000_println!("(IGNORING) {:?}", e);
                }
                link_check = !self.hw.mac.get_link_status;
            }
            _ => {
                return Err("Unsupported hardware".to_string());
            }
        }

        // /* Check for thermal downshift or shutdown */
        // if (hw->mac.type == e1000_i350) {
        // 	thstat = E1000_READ_REG(hw, E1000_THSTAT);
        // 	ctrl = E1000_READ_REG(hw, E1000_CTRL_EXT);
        // }
        if self.is_mac(MacType::Mac_i350) {
            return Err("Unsupported hardware".to_string());
        }

        // /* Now check for a transition */
        // if (link_check && (adapter->link_active == 0)) {
        // 	e1000_get_speed_and_duplex(hw, &adapter->link_speed,
        // 	    &adapter->link_duplex);

        e1000_println!(
            "link_check: {:?}, link_active: {:?}",
            link_check,
            self.link_active
        );

        if link_check && !self.link_active {
            try!(self.get_speed_and_duplex());

            /* Check if we must disable SPEED_MODE bit on PCI-E */
            // 	if ((adapter->link_speed != SPEED_1000) &&
            // 	    ((hw->mac.type == e1000_82571) ||
            // 	    (hw->mac.type == e1000_82572))) {
            // 		int tarc0;
            // 		tarc0 = E1000_READ_REG(hw, E1000_TARC(0));
            // 		tarc0 &= ~TARC_SPEED_MODE_BIT;
            // 		E1000_WRITE_REG(hw, E1000_TARC(0), tarc0);
            // 	}
            if self.link_speed != SPEED_1000
                && (self.is_mac(MacType::Mac_82571) || self.is_mac(MacType::Mac_82572))
            {
                incomplete_return!();
            }
            // 	if (bootverbose)
            // 		device_printf(dev, "Link is up %d Mbps %s\n",
            // 		    adapter->link_speed,
            // 		    ((adapter->link_duplex == FULL_DUPLEX) ?
            // 		    "Full Duplex" : "Half Duplex"));
            // 	adapter->link_active = 1;
            // 	adapter->smartspeed = 0;
            e1000_println!("Link is up {} Mbps", self.link_speed);
            self.link_active = true;
            self.smartspeed = 0;

            // 	if_setbaudrate(ifp, adapter->link_speed * 1000000);
            self.ifnet.set_baudrate(self.link_speed as u64 * 1_000_000);

            // 	if ((ctrl & E1000_CTRL_EXT_LINK_MODE_GMII) &&
            // 	    (thstat & E1000_THSTAT_LINK_THROTTLE))
            // 		device_printf(dev, "Link: thermal downshift\n");
            // 	/* Delay Link Up for Phy update */
            // 	if (((hw->mac.type == e1000_i210) ||
            // 	    (hw->mac.type == e1000_i211)) &&
            // 	    (hw->phy.id == I210_I_PHY_ID))
            // 		msec_delay(I210_LINK_DELAY);
            // 	/* Reset if the media type changed. */
            // 	if ((hw->dev_spec._82575.media_changed) &&
            // 		(adapter->hw.mac.type >= igb_mac_min)) {
            // 		hw->dev_spec._82575.media_changed = false;
            // 		adapter->flags |= IGB_MEDIA_RESET;
            // 		em_reset(ctx);
            // 	}
            if btst!(ctrl, E1000_CTRL_EXT_LINK_MODE_GMII)
                && btst!(thstat, E1000_THSTAT_LINK_THROTTLE)
            {
                incomplete_return!();
            }
            if self.is_macs(&[MacType::Mac_i210, MacType::Mac_i211])
                && self.hw.phy.id == I210_I_PHY_ID
            {
                incomplete_return!();
            }
            if self.hw.mac.mac_type >= MacType::IGB_MAC_MIN {
                incomplete_return!();
            }
            // 	iflib_link_state_change(ctx, LINK_STATE_UP, ifp->if_baudrate);
            // 	printf("Link state changed to up\n");
            self.iflib
                .link_state_change(LINK_STATE_UP as i32, self.ifnet.if_baudrate);
            e1000_println!("Link state changed to up");

        // } else if (!link_check && (adapter->link_active == 1)) {
        } else if !link_check && self.link_active {
            // 	if_setbaudrate(ifp, 0);
            // 	adapter->link_speed = 0;
            // 	adapter->link_duplex = 0;
            // 	if (bootverbose)
            // 		device_printf(dev, "Link is Down\n");
            // 	adapter->link_active = 0;
            // 	iflib_link_state_change(ctx, LINK_STATE_DOWN, ifp->if_baudrate);
            // 	printf("link state changed to down\n");
            // }
            self.ifnet.set_baudrate(0);
            self.link_speed = 0;
            self.link_duplex = 0;
            self.link_active = false;
            self.iflib
                .link_state_change(LINK_STATE_DOWN as i32, self.ifnet.if_baudrate);
            e1000_println!("Link state changed to down");
        }
        // em_update_stats_counters(adapter);
        self.update_stats_counters();

        /* Reset LAA into RAR[0] on 82571 */
        // if ((adapter->hw.mac.type == e1000_82571) &&
        //     e1000_get_laa_state_82571(&adapter->hw))
        // 	e1000_rar_set(&adapter->hw, adapter->hw.mac.addr, 0);

        // if (adapter->hw.mac.type < em_mac_min)
        // 	lem_smartspeed(adapter);
        if self.hw.mac.mac_type < MacType::EM_MAC_MIN {
            match self.lem_smartspeed() {
                Ok(()) => (),
                Err(e) => eprintln!(e),
            }
        }

        // E1000_WRITE_REG(&adapter->hw, E1000_IMS, EM_MSIX_LINK | E1000_IMS_LSC);
        do_write_register(self, E1000_IMS, EM_MSIX_LINK | E1000_IMS_LSC);
        Ok(())
    }
    pub fn get_speed_and_duplex(&mut self) -> AdResult {
        e1000_println!();
        self.hw
            .mac
            .ops
            .get_link_up_info
            .ok_or("No function".to_string())
            .and_then(|f| {
                let mut link_speed = 0;
                let mut link_duplex = 0;
                try!(f(self, &mut link_speed, &mut link_duplex));
                self.link_speed = link_speed;
                self.link_duplex = link_duplex;
                Ok(())
            })
    }
    pub fn cfg_on_link_up(&mut self) -> AdResult {
        if let Some(f) = self.hw.phy.ops.cfg_on_link_up {
            e1000_println!();
            try!(f(self));
        } else {
            e1000_println!("(IGNORE) No function: cfg_on_link_up()");
        }
        Ok(())
    }
    pub fn add_hw_stats(&mut self) {
        // Only sysctl stuff here
        e1000_println!("Only sysctl stuff - ignore this function for now");
        // incomplete!();
    }
    pub fn rar_set(&mut self, addr: &[u8], index: usize) -> AdResult {
        e1000_println!();
        if let Some(f) = self.hw.mac.ops.rar_set {
            try!(f(self, addr, index));
        } else {
            return Err("No function: rar_set()".to_string());
        }
        Ok(())
    }
    pub fn get_hw_control(&mut self) {
        e1000_println!();

        // if (adapter->vf_ifp)
        //     return;

        if self.vf_ifp != 0 {
            return;
        }

        // if (adapter->hw.mac.type == e1000_82573) {
        //     swsm = E1000_READ_REG(&adapter->hw, E1000_SWSM);
        //     E1000_WRITE_REG(&adapter->hw, E1000_SWSM,
        // 	            swsm | E1000_SWSM_DRV_LOAD);
        //     return;
        // }
        // /* else */
        // ctrl_ext = E1000_READ_REG(&adapter->hw, E1000_CTRL_EXT);
        // E1000_WRITE_REG(&adapter->hw, E1000_CTRL_EXT,
        //                 ctrl_ext | E1000_CTRL_EXT_DRV_LOAD);
        if self.is_mac(MacType::Mac_82573) {
            panic!("Unsupported hardware");
        }
        let ctrl_ext = do_read_register(self, E1000_CTRL_EXT);
        do_write_register(self, E1000_CTRL_EXT, ctrl_ext | E1000_CTRL_EXT_DRV_LOAD);
    }
    pub fn led_off(&mut self) -> AdResult {
        e1000_println!();
        if let Some(f) = self.hw.mac.ops.led_off {
            try!(f(self));
        }
        Ok(())
    }
    pub fn cleanup_led(&mut self) -> AdResult {
        e1000_println!();
        if let Some(f) = self.hw.mac.ops.cleanup_led {
            try!(f(self));
        }
        Ok(())
    }
}

impl Ifdi for Adapter {
    fn init_pre(
        &mut self,
        dev: PciDevice,
        iflib: IfLib,
        iflib_shared: IfLibShared,
        media: IfMedia,
        ifnet: IfNet,
    ) -> AdResult {
        e1000_println!();
        self.dev = dev;
        self.iflib = iflib;
        self.iflib_shared = iflib_shared;
        self.ifmedia = media;
        self.ifnet = ifnet;
        Ok(())
    }
    fn init(&mut self) -> Result<(), String> {
        e1000_println!();
        //      struct adapter *adapter = iflib_get_softc(ctx);
        // 	struct ifnet *ifp = iflib_get_ifp(ctx);
        // 	struct em_tx_queue *tx_que;
        // 	int i;
        // 	INIT_DEBUGOUT("em_if_init: begin");

        /* Get the latest mac address, User can use a LAA */
        // 	bcopy(if_getlladdr(ifp), adapter->hw.mac.addr,
        // 	    ETHER_ADDR_LEN);
        let mac_addr: [u8; 6] = self.ifnet.lladdr();
        self.hw.mac.addr = mac_addr;

        /* Put the address into the Receive Address Array */
        // e1000_rar_set(&adapter->hw, adapter->hw.mac.addr, 0);
        try!(self.rar_set(&mac_addr, 0));

        /*
         * With the 82571 adapter, RAR[0] may be overwritten
         * when the other port is reset, we make a duplicate
         * in RAR[14] for that eventuality, this assures
         * the interface continues to function.
         */
        // 	if (adapter->hw.mac.type == e1000_82571) {
        // 		e1000_set_laa_state_82571(&adapter->hw, TRUE);
        // 		e1000_rar_set(&adapter->hw, adapter->hw.mac.addr,
        // 		    E1000_RAR_ENTRIES - 1);
        // 	}
        if self.hw.mac.mac_type == MacType::Mac_82571 {
            return Err("Unsupported hardware".to_string());
        }

        /* Initialize the hardware */
        // 	em_reset(ctx);
        // 	em_if_update_admin_status(ctx);
        try!(self.reset());
        try!(self.update_admin_status());

        // 	for (i = 0, tx_que = adapter->tx_queues; i < adapter->tx_num_queues; i++, tx_que++) {
        // 		struct tx_ring *txr = &tx_que->txr;

        // 		txr->tx_rs_cidx = txr->tx_rs_pidx = txr->tx_cidx_processed = 0;
        // 	}
        for txq in &mut self.tx_queues.iter_mut() {
            let mut txr: &mut TxRing = &mut txq.txr;
            txr.tx_rs_cidx = 0;
            txr.tx_rs_pidx = 0;
            txr.tx_cidx_processed = 0;
        }

        /* Setup VLAN support, basic and offload if available */
        // 	E1000_WRITE_REG(&adapter->hw, E1000_VET, ETHERTYPE_VLAN);
        do_write_register(self, E1000_VET, ETHERTYPE_VLAN);

        /* Clear bad data from Rx FIFOs */
        // 	if (adapter->hw.mac.type >= igb_mac_min)
        // 		e1000_rx_fifo_flush_82575(&adapter->hw);
        if self.hw.mac.mac_type >= MacType::IGB_MAC_MIN {
            return Err("Unsupported hardware".to_string());
        }

        /* Configure for OS presence */
        // em_init_manageability(adapter);
        self.init_manageability();

        /* Prepare transmit descriptors and buffers */
        // 	em_initialize_transmit_unit(ctx);
        self.initialize_transmit_unit();

        /* Setup Multicast table */
        // 	em_if_multi_set(ctx);
        try!(self.multi_set());

        /*
         * Figure out the desired mbuf
         * pool for doing jumbos
         */
        // 	if (adapter->hw.mac.max_frame_size <= 2048)
        // 		adapter->rx_mbuf_sz = MCLBYTES;
        // #ifndef CONTIGMALLOC_WORKS
        // 	else
        // 		adapter->rx_mbuf_sz = MJUMPAGESIZE;
        // #else
        // 	else if (adapter->hw.mac.max_frame_size <= 4096)
        // 		adapter->rx_mbuf_sz = MJUMPAGESIZE;
        // 	else
        // 		adapter->rx_mbuf_sz = MJUM9BYTES;
        // #endif

        self.rx_mbuf_sz = match self.hw.mac.max_frame_size {
            x if x <= 2048 => kernel::sys::iflib_sys::MCLBYTES,
            _ => return Err("Only frame size <= 2048 is supported".to_string()),
        };

        // 	em_initialize_receive_unit(ctx);
        try!(self.initialize_receive_unit());

        // 	/* Use real VLAN Filter support? */
        // 	if (if_getcapenable(ifp) & IFCAP_VLAN_HWTAGGING) {
        // 		if (if_getcapenable(ifp) & IFCAP_VLAN_HWFILTER)
        // 			/* Use real VLAN Filter support */
        // 			em_setup_vlan_hw_support(adapter);
        // 		else {
        // 			u32 ctrl;
        // 			ctrl = E1000_READ_REG(&adapter->hw, E1000_CTRL);
        // 			ctrl |= E1000_CTRL_VME;
        // 			E1000_WRITE_REG(&adapter->hw, E1000_CTRL, ctrl);
        // 		}
        // 	}
        if self.ifnet.capenable() & IFCAP_VLAN_HWTAGGING != 0 {
            if self.ifnet.capenable() & IFCAP_VLAN_HWFILTER != 0 {
                self.setup_vlan_hw_support();
            } else {
                let mut ctrl: u32 = do_read_register(self, E1000_CTRL);
                ctrl |= E1000_CTRL_VME;
                do_write_register(self, E1000_CTRL, ctrl);
            }
        }

        /* Don't lose promiscuous settings */
        // em_if_set_promisc(ctx, IFF_PROMISC);
        try!(self.set_promisc(IFF_PROMISC));

        // e1000_clear_hw_cntrs_base_generic(&adapter->hw);
        e1000_mac::clear_hw_cntrs_base_generic(self);

        /* MSI/X configuration for 82574 */
        // if (adapter->hw.mac.type == e1000_82574) {
        //     int tmp = E1000_READ_REG(&adapter->hw, E1000_CTRL_EXT);

        //     tmp |= E1000_CTRL_EXT_PBA_CLR;
        //     E1000_WRITE_REG(&adapter->hw, E1000_CTRL_EXT, tmp);
        //     /* Set the IVAR - interrupt vector routing. */
        //     E1000_WRITE_REG(&adapter->hw, E1000_IVAR, adapter->ivars);
        // } else if (adapter->intr_type == IFLIB_INTR_MSIX) /* Set up queue routing */
        //     igb_configure_queues(adapter);
        // ^^^ Nothing for 82545
        if self.is_mac(MacType::Mac_82574)
            || self.iflib_shared.isc_intr == iflib_intr_mode_t::IFLIB_INTR_MSIX
        {
            return Err("Unsupported hardware".to_string());
        } else if self.iflib_shared.isc_intr == iflib_intr_mode_t::IFLIB_INTR_MSIX {
            return Err("MSIX on other than 82574 not implemented".to_string());
        }

        /* this clears any pending interrupts */
        // E1000_READ_REG(&adapter->hw, E1000_ICR);
        // E1000_WRITE_REG(&adapter->hw, E1000_ICS, E1000_ICS_LSC);
        do_read_register(self, E1000_ICR);
        do_write_register(self, E1000_ICS, E1000_ICS_LSC);

        // /* AMT based hardware can now take control from firmware */
        // if (adapter->has_manage && adapter->has_amt)
        //     em_get_hw_control(adapter);
        if self.has_manage && self.has_amt {
            self.get_hw_control();
        }

        /* Set Energy Efficient Ethernet */
        // if (adapter->hw.mac.type >= igb_mac_min &&
        //     adapter->hw.phy.media_type == e1000_media_type_copper) {
        //     if (adapter->hw.mac.type == e1000_i354)
        // 	e1000_set_eee_i354(&adapter->hw, TRUE, TRUE);
        //     else
        // 	e1000_set_eee_i350(&adapter->hw, TRUE, TRUE);
        // }
        if self.hw.mac.mac_type >= MacType::IGB_MAC_MIN && self.is_copper() {
            return Err("Unsupported hardware".to_string());
        }

        // e1000_println!("adapter: {:?}", self);

        Ok(())
    }
    fn attach_pre(&mut self) -> Result<(), String> {
        e1000_println!();

        self.tx_process_limit = self.iflib_shared.tx_process_limit() as u32;

        // Set mac_type and pci device info
        try!(self.identify_hardware());

        // Setup up stuff in shared C struct for iflib
        try!(self.setup_shared_context());

        /* Setup PCI resources */
        try!(self.allocate_pci_resources());

        /*
        	** For ICH8 and family we need to
        	** map the flash memory, and this
        	** must happen after the MAC is
        	** identified
         */
        // if ((hw->mac.type == e1000_ich8lan) ||
        //     (hw->mac.type == e1000_ich9lan) ||
        //     (hw->mac.type == e1000_ich10lan) ||
        //     (hw->mac.type == e1000_pchlan) ||
        //     (hw->mac.type == e1000_pch2lan) ||
        //     (hw->mac.type == e1000_pch_lpt)) {
        // 	int rid = EM_BAR_TYPE_FLASH;
        // 	adapter->flash = bus_alloc_resource_any(dev,
        // 	    SYS_RES_MEMORY, &rid, RF_ACTIVE);
        // 	if (adapter->flash == NULL) {
        // 		device_printf(dev, "Mapping of Flash failed\n");
        // 		error = ENXIO;
        // 		goto err_pci;
        // 	}
        // 	/* This is used in the shared code */
        // 	hw->flash_address = (u8 *)adapter->flash;
        // 	adapter->osdep.flash_bus_space_tag =
        // 	    rman_get_bustag(adapter->flash);
        // 	adapter->osdep.flash_bus_space_handle =
        // 	    rman_get_bushandle(adapter->flash);
        // }
        let ich8_macs = [
            MacType::Mac_ich8lan,
            MacType::Mac_ich9lan,
            MacType::Mac_ich10lan,
            MacType::Mac_pchlan,
            MacType::Mac_pch2lan,
            MacType::Mac_pch_lpt,
        ];
        if self.is_macs(&ich8_macs) {
            // We are here for UK Dell machine
            e1000_println!("<<<<<<<<<<<<<<<<<<<<< ICH8 >>>>>>>>>>>>>>>>>>>>>");

            let mut rid = EM_BAR_TYPE_FLASH as i32;
            self.flash =
                self.dev
                    .bus_alloc_resource_any(SYS_RES_MEMORY as i32, &mut rid, RF_ACTIVE);
            if self.flash.is_none() {
                return Err("Unable to allocate bus resource: flash".to_string());
            }

            self.hw.flash_address = self.flash.as_ref().unwrap().inner.as_ptr() as *mut u8;
            self.osdep.flash_bus_space_tag = self.flash.as_ref().unwrap().rman_get_bustag();
            self.osdep.flash_bus_space_handle = self.flash.as_ref().unwrap().rman_get_bushandle();
        }
        /*
         ** In the new SPT device flash is not  a
         ** separate BAR, rather it is also in BAR0,
         ** so use the same tag and an offset handle for the
         ** FLASH read/write macros in the shared code.
         */
        // else if (hw->mac.type >= e1000_pch_spt) {
        // 	adapter->osdep.flash_bus_space_tag =
        // 	    adapter->osdep.mem_bus_space_tag;
        // 	adapter->osdep.flash_bus_space_handle =
        // 	    adapter->osdep.mem_bus_space_handle
        // 	    + E1000_FLASH_BASE_ADDR;
        // }
        else if self.hw.mac.mac_type >= MacType::Mac_pch_spt {
            e1000_println!("<<<<<<<<<<<<<<<<<<<<< SPT >>>>>>>>>>>>>>>>>>>>>");
            self.osdep.flash_bus_space_tag = self.osdep.mem_bus_space_tag;
            self.osdep.flash_bus_space_handle =
                self.osdep.mem_bus_space_handle + E1000_FLASH_BASE_ADDR as u64;
        } else {
            e1000_println!("<<<<<<<<<<<<<<<<<<<<< ??? >>>>>>>>>>>>>>>>>>>>>");
        }

        // /* Do Shared Code initialization */
        try!(self.setup_init_functions(true));

        try!(self.setup_msix());

        // e1000_get_bus_info(hw);
        // if let Some(f) = self.hw.mac.ops.get_bus_info {
        //     try!(f(self))
        // } else {
        //     eprintln!("No get_bus_info() function");
        // }
        try!(
            self.hw
                .mac
                .ops
                .get_bus_info
                .ok_or("No function: get_bus_info".to_string())
                .and_then(|f| f(self))
        );

        // hw->mac.autoneg = DO_AUTO_NEG;
        // hw->phy.autoneg_wait_to_complete = FALSE;
        // hw->phy.autoneg_advertised = AUTONEG_ADV_DEFAULT;

        self.hw.mac.autoneg = DO_AUTO_NEG > 0;
        self.hw.phy.autoneg_wait_to_complete = false;
        self.hw.phy.autoneg_advertised = AUTONEG_ADV_DEFAULT as u16;

        // if (adapter->hw.mac.type < em_mac_min) {
        // 	e1000_init_script_state_82541(&adapter->hw, TRUE);
        // 	e1000_set_tbi_compatibility_82543(&adapter->hw, TRUE);
        // }
        // /* Copper options */
        // if (hw->phy.media_type == e1000_media_type_copper) {
        // 	hw->phy.mdix = AUTO_ALL_MODES;
        // 	hw->phy.disable_polarity_correction = FALSE;
        // 	hw->phy.ms_type = EM_MASTER_SLAVE;
        // }

        if self.hw.mac.mac_type < MacType::EM_MAC_MIN {
            // 82545 < 82547
            e1000_82541::init_script_state(self, true);
            e1000_82543::set_tbi_compatibility(self, true);
        }

        /* Copper options */
        if self.hw.phy.media_type == MediaType::Copper {
            self.hw.phy.mdix = AUTO_ALL_MODES as u8;
            self.hw.phy.disable_polarity_correction = false;
            self.hw.phy.ms_type = MsType::EM_MASTER_SLAVE;
        }

        /*
         * Set the frame limits assuming
         * standard ethernet sized frames.
         */
        // scctx->isc_max_frame_size = adapter->hw.mac.max_frame_size =
        //     ETHERMTU + ETHER_HDR_LEN + ETHERNET_FCS_SIZE;

        self.iflib_shared.isc_max_frame_size = (kernel::sys::iflib_sys::ETHERMTU
            + kernel::sys::iflib_sys::ETHER_HDR_LEN
            + ETHERNET_FCS_SIZE) as u16;
        self.hw.mac.max_frame_size = kernel::sys::iflib_sys::ETHERMTU
            + kernel::sys::iflib_sys::ETHER_HDR_LEN
            + ETHERNET_FCS_SIZE;

        /*
         * This controls when hardware reports transmit completion
         * status.
         */
        // hw->mac.report_tx_early = 1;
        self.hw.mac.report_tx_early = true;

        /* Allocate multicast array memory. */
        // adapter->mta = malloc(sizeof(u8) * ETH_ADDR_LEN *
        //                       MAX_NUM_MULTICAST_ADDRESSES, M_DEVBUF, M_NOWAIT);
        // if (adapter->mta == NULL) {
        // 	device_printf(dev, "Can not allocate multicast setup array\n");
        // 	error = ENOMEM;
        // 	goto err_late;
        // }
        self.mta = Box::new(
            [0u8;
                (kernel::sys::iflib_sys::ETHER_HDR_LEN * ::sys::e1000::MAX_NUM_MULTICAST_ADDRESSES)
                    as usize],
        );

        /* Check SOL/IDER usage */
        // if (e1000_check_reset_block(hw))
        // 	device_printf(dev, "PHY reset is blocked"
        // 		      " due to SOL/IDER session.\n");
        match self.check_reset_block() {
            Ok(true) => e1000_println!("PHY reset is blocked"),
            Ok(false) => e1000_println!("PHY reset is not blocked"),
            Err(e) => eprintln!(e),
        }

        /* Sysctl for setting Energy Efficient Ethernet */
        unsafe {
            // access to union field is unsafe
            self.hw.dev_spec.ich8lan.eee_disable = true;
        }

        /*
         * Start from a known state, this is
         * important in reading the nvm and
         * mac from that.
         */
        try!(self.mac_reset_hw());

        /* Make sure we have a good EEPROM before we read from it */
        // if (e1000_validate_nvm_checksum(hw) < 0) {
        // 	if (e1000_validate_nvm_checksum(hw) < 0) {
        // 		device_printf(dev,
        // 		    "The EEPROM Checksum Is Not Valid\n");
        // 		error = EIO;
        // 		goto err_late;
        // 	}
        // }
        if self.validate_nvm_checksum().is_err() {
            /*
             * Some PCI-E parts fail the first check due to
             * the link being in sleep state, call it again,
             * if it fails a second time its a real issue.
             */
            try!(self.validate_nvm_checksum());
        }

        /* Copy the permanent MAC address out of the EEPROM */
        // if (e1000_read_mac_addr(hw) < 0) {
        // 	device_printf(dev, "EEPROM read error while reading MAC"
        // 		      " address\n");
        // 	error = EIO;
        // 	goto err_late;
        // }
        try!(self.mac_read_mac_addr());

        // if (!em_is_valid_ether_addr(hw->mac.addr)) {
        //     device_printf(dev, "Invalid MAC address\n");
        //     error = EIO;
        //     goto err_late;
        // }
        try!(is_valid_ether_addr(&self.hw.mac.addr));

        /* Disable ULP support */
        // e1000_disable_ulp_lpt_lp(hw, TRUE);
        if let Err(e) = e1000_ich8lan::disable_ulp_lpt_lp(self, true) {
            eprintln!("(NOT FATAL) {:?}", e);
        }

        /*
         * Get Wake-on-Lan and Management info for later use
         */
        // em_get_wakeup(ctx);
        try!(self.get_wakeup());

        // iflib_set_mac(ctx, hw->mac.addr);
        self.iflib.set_mac(&self.hw.mac.addr);

        // println!("{:?}", self);

        Ok(())
        // Err("Fail on purpose".into())
    }

    fn tx_queues_alloc(
        &mut self,
        vaddrs: *mut caddr_t,
        paddrs: *mut u64,
        ntxqs: usize,
        ntxqsets: usize,
    ) -> Result<(), String> {
        e1000_println!();

        // 	MPASS(adapter->tx_num_queues > 0);
        // 	MPASS(adapter->tx_num_queues == ntxqsets);
        // #define tx_num_queues shared->isc_ntxqsets
        // #define rx_num_queues shared->isc_nrxqsets
        assert!(self.iflib_shared.isc_ntxqsets > 0);
        assert!(self.iflib_shared.isc_ntxqsets == ntxqsets as i32);

        unsafe {
            e1000_println!("vaddrs[0]: {:p}, paddrs[0]: 0x{:x}", *vaddrs, *paddrs);
        }
        e1000_println!("ntxqs: {:?}, ntxqsets: {:?}", ntxqs, ntxqsets);

        /* First allocate the top level queue structs */
        // 	if (!(adapter->tx_queues =
        // 	    (struct em_tx_queue *) malloc(sizeof(struct em_tx_queue) *
        // 	    adapter->tx_num_queues, M_DEVBUF, M_NOWAIT | M_ZERO))) {
        // 		device_printf(iflib_get_dev(ctx), "Unable to allocate queue memory\n");
        // 		return(ENOMEM);
        // 	}

        // 	for (i = 0, que = adapter->tx_queues; i < adapter->tx_num_queues; i++, que++) {
        // 		/* Set up some basics */

        // 		struct tx_ring *txr = &que->txr;
        // 		txr->adapter = que->adapter = adapter;
        // 		que->me = txr->me =  i;

        // 		/* Allocate report status array */
        // 		if (!(txr->tx_rsq = (qidx_t *) malloc(sizeof(qidx_t) * scctx->isc_ntxd[0], M_DEVBUF, M_NOWAIT | M_ZERO))) {
        // 			device_printf(iflib_get_dev(ctx), "failed to allocate rs_idxs memory\n");
        // 			error = ENOMEM;
        // 			goto fail;
        // 		}
        // 		for (j = 0; j < scctx->isc_ntxd[0]; j++)
        // 			txr->tx_rsq[j] = QIDX_INVALID;
        // 		/* get the virtual and physical address of the hardware queues */
        // 		txr->tx_base = (struct e1000_tx_desc *)vaddrs[i*ntxqs];
        // 		txr->tx_paddr = paddrs[i*ntxqs];
        // 	}

        // 	device_printf(iflib_get_dev(ctx), "allocated for %d tx_queues\n", adapter->tx_num_queues);
        // 	return (0);
        // fail:
        // 	em_if_queues_free(ctx);
        // 	return (error);

        /* First allocate the top level queue structs */

        let count = ntxqsets * ntxqs;
        let vaddrs_slice: &[caddr_t] = unsafe { kernel::slice::from_raw_parts(vaddrs, count) };
        let paddrs_slice: &[u64] = unsafe { kernel::slice::from_raw_parts(paddrs, count) };

        let mut queues = vec![];
        for i in 0..ntxqsets {
            let mut queue = TxQueue::default();
            queue.me = i as u32;

            /* Set up some basics */
            {
                let mut ring: &mut TxRing = &mut queue.txr;
                ring.me = i as u8;

                /* Allocate report status array */
                let mut rsq = vec![];
                for j in 0..self.iflib_shared.isc_ntxd[0] {
                    rsq.push(QIDX_INVALID as u16);
                }
                ring.tx_rsq = rsq.into_boxed_slice();

                /* get the virtual and physical address of the hardware queues */
                ring.tx_base = vaddrs_slice[(i * ntxqs) as usize] as *mut e1000_tx_desc;
                ring.tx_paddr = paddrs_slice[(i * ntxqs) as usize];
            }
            // e1000_println!("{:?}", queue);

            queues.push(queue);
        }
        self.tx_queues = queues.into_boxed_slice();

        e1000_println!("allocated for {} tx_queues", self.iflib_shared.isc_ntxqsets);
        // e1000_println!("{:?}", self.tx_queues);

        // incomplete!();
        // Err("Fail on purpose".to_string())
        Ok(())
    }

    fn rx_queues_alloc(
        &mut self,
        vaddrs: *mut caddr_t,
        paddrs: *mut u64,
        nrxqs: usize,
        nrxqsets: usize,
    ) -> Result<(), String> {
        e1000_println!();

        assert!(self.iflib_shared.isc_nrxqsets > 0);
        assert!(self.iflib_shared.isc_nrxqsets == nrxqsets as i32);

        unsafe {
            e1000_println!("vaddrs[0]: {:p}, paddrs[0]: 0x{:x}", *vaddrs, *paddrs);
        }
        e1000_println!("nrxqs: {:?}, nrxqsets: {:?}", nrxqs, nrxqsets);

        /* First allocate the top level queue structs */
        // if (!(adapter->rx_queues =
        //     (struct em_rx_queue *) malloc(sizeof(struct em_rx_queue) *
        //     adapter->rx_num_queues, M_DEVBUF, M_NOWAIT | M_ZERO))) {
        // 	device_printf(iflib_get_dev(ctx), "Unable to allocate queue memory\n");
        // 	error = ENOMEM;
        // 	goto fail;
        // }

        // for (i = 0, que = adapter->rx_queues; i < nrxqsets; i++, que++) {
        // 	/* Set up some basics */
        // 	struct rx_ring *rxr = &que->rxr;
        // 	rxr->adapter = que->adapter = adapter;
        // 	rxr->que = que;
        // 	que->me = rxr->me =  i;

        // 	/* get the virtual and physical address of the hardware queues */
        // 	rxr->rx_base = (union e1000_rx_desc_extended *)vaddrs[i*nrxqs];
        // 	rxr->rx_paddr = paddrs[i*nrxqs];
        // }

        let count = nrxqsets * nrxqs;
        let vaddrs_slice: &[caddr_t] = unsafe { kernel::slice::from_raw_parts(vaddrs, count) };
        let paddrs_slice: &[u64] = unsafe { kernel::slice::from_raw_parts(paddrs, count) };

        /* First allocate the top level queue structs */
        let mut queues = vec![];
        for i in 0..nrxqsets {
            let mut queue = RxQueue::default();
            queue.me = i as u32;

            /* Set up some basics */
            {
                let mut ring: &mut RxRing = &mut queue.rxr;
                ring.me = i as u32;

                /* get the virtual and physical address of the hardware queues */
                ring.rx_base = vaddrs_slice[(i * nrxqs) as usize] as *mut e1000_rx_desc_extended;
                ring.rx_paddr = paddrs_slice[(i * nrxqs) as usize];
            }
            e1000_println!("{:?}", queue);

            queues.push(queue);
        }
        self.rx_queues = queues.into_boxed_slice();

        e1000_println!("allocated for {} rx_queues", self.iflib_shared.isc_nrxqsets);
        // e1000_println!("{:?}", self.rx_queues);

        // incomplete!();
        // Err("Fail on purpose".to_string())
        Ok(())
    }

    fn enable_intr(&mut self) {
        e1000_println!();
        // u32 ims_mask = IMS_ENABLE_MASK;
        // if (hw->mac.type == e1000_82574) {
        // 	E1000_WRITE_REG(hw, EM_EIAC, EM_MSIX_MASK);
        // 	ims_mask |= adapter->ims;
        // } else if (adapter->intr_type == IFLIB_INTR_MSIX && hw->mac.type >= igb_mac_min)  {
        // 	u32 mask = (adapter->que_mask | adapter->link_mask);

        // 	E1000_WRITE_REG(&adapter->hw, E1000_EIAC, mask);
        // 	E1000_WRITE_REG(&adapter->hw, E1000_EIAM, mask);
        // 	E1000_WRITE_REG(&adapter->hw, E1000_EIMS, mask);
        // 	ims_mask = E1000_IMS_LSC;
        // }
        // E1000_WRITE_REG(hw, E1000_IMS, ims_mask);

        // No MSIX on our devices (82545,spt,lpt)

        if self.hw.mac.mac_type == MacType::Mac_82574 {
            unsupported!();
        } else if self.iflib_shared.isc_intr == iflib_intr_mode_t::IFLIB_INTR_MSIX
            && self.hw.mac.mac_type >= MacType::IGB_MAC_MIN
        {
            unsupported!();
        }

        e1000_println!("Using legacy interrupt");

        do_write_register(self, E1000_IMS, IMS_ENABLE_MASK);
    }

    fn disable_intr(&mut self) {
        e1000_println!();
        // if (adapter->intr_type == IFLIB_INTR_MSIX) {
        // 	if (hw->mac.type >= igb_mac_min)
        // 		E1000_WRITE_REG(&adapter->hw, E1000_EIMC, ~0);
        // 	E1000_WRITE_REG(&adapter->hw, E1000_EIAC, 0);
        // }
        // E1000_WRITE_REG(&adapter->hw, E1000_IMC, 0xffffffff);

        if self.iflib_shared.isc_intr == iflib_intr_mode_t::IFLIB_INTR_MSIX {
            unsupported!();
        }

        // No MSIX on our device
        do_write_register(self, E1000_IMC, 0xffffffff);
    }

    fn timer(&mut self, qid: u16) {
        e1000_println!("qid: {}", qid);
        // 	struct adapter *adapter = iflib_get_softc(ctx);
        // struct em_rx_queue *que;
        // int i;
        // int trigger = 0;

        // if (qid != 0)
        // 	return;
        if qid != 0 {
            return;
        }

        // iflib_admin_intr_deferred(ctx);
        self.iflib.admin_intr_deferred();

        /* Mask to use in the irq trigger */
        // if (adapter->intr_type == IFLIB_INTR_MSIX) {
        // 	for (i = 0, que = adapter->rx_queues; i < adapter->rx_num_queues; i++, que++)
        // 		trigger |= que->eims;
        // } else {
        // 	trigger = E1000_ICS_RXDMT0;
        // }
        if self.iflib_shared.isc_intr == iflib_intr_mode_t::IFLIB_INTR_MSIX {
            unsupported!();
        }
        // what they plan to do with 'trigger' here??
    }

    fn get_counter(&mut self, cnt: IftCounter) -> u64 {
        // e1000_verbose_println!("{:?}", cnt);
        // struct adapter *adapter = iflib_get_softc(ctx);
        // struct ifnet *ifp = iflib_get_ifp(ctx);

        // switch (cnt) {
        // case IFCOUNTER_COLLISIONS:
        // 	return (adapter->stats.colc);
        // case IFCOUNTER_IERRORS:
        // 	return (adapter->dropped_pkts + adapter->stats.rxerrc +
        // 	    adapter->stats.crcerrs + adapter->stats.algnerrc +
        // 	    adapter->stats.ruc + adapter->stats.roc +
        // 	    adapter->stats.mpc + adapter->stats.cexterr);
        // case IFCOUNTER_OERRORS:
        // 	return (adapter->stats.ecol + adapter->stats.latecol +
        // 	    adapter->watchdog_events);
        // default:
        // 	return (if_get_counter_default(ifp, cnt));
        // }
        match cnt {
            IftCounter::COLLISIONS => self.stats.colc,
            IftCounter::IERRORS => {
                self.dropped_pkts + self.stats.rxerrc + self.stats.crcerrs + self.stats.algnerrc
                    + self.stats.ruc + self.stats.roc + self.stats.mpc
                    + self.stats.cexterr
            }
            IftCounter::OERRORS => self.stats.ecol + self.stats.latecol + self.watchdog_events,
            _ => self.ifnet.counter_default(cnt),
        }
    }

    fn media_status(&mut self, ifmr: &mut IfMediaReq) {
        e1000_println!();

        // struct adapter *adapter = iflib_get_softc(ctx);
        // u_char fiber_type = IFM_1000_SX;

        // INIT_DEBUGOUT("em_if_media_status: begin");

        // iflib_admin_intr_deferred(ctx);
        self.iflib.admin_intr_deferred();

        // ifmr->ifm_status = IFM_AVALID;
        // ifmr->ifm_active = IFM_ETHER;
        ifmr.ifm_status = IFM_AVALID as i32;
        ifmr.ifm_active = IFM_ETHER as i32;

        // if (!adapter->link_active) {
        // 	return;
        // }
        if !self.link_active {
            e1000_println!("link not active - returning early");
            return;
        }

        // ifmr->ifm_status |= IFM_ACTIVE;
        ifmr.ifm_status |= IFM_ACTIVE as i32;

        // if ((adapter->hw.phy.media_type == e1000_media_type_fiber) ||
        //     (adapter->hw.phy.media_type == e1000_media_type_internal_serdes)) {
        // 	if (adapter->hw.mac.type == e1000_82545)
        // 		fiber_type = IFM_1000_LX;
        // 	ifmr->ifm_active |= fiber_type | IFM_FDX;
        // } else {
        // 	switch (adapter->link_speed) {
        // 	case 10:
        // 		ifmr->ifm_active |= IFM_10_T;
        // 		break;
        // 	case 100:
        // 		ifmr->ifm_active |= IFM_100_TX;
        // 		break;
        // 	case 1000:
        // 		ifmr->ifm_active |= IFM_1000_T;
        // 		break;
        // 	}
        // 	if (adapter->link_duplex == FULL_DUPLEX)
        // 		ifmr->ifm_active |= IFM_FDX;
        // 	else
        // 		ifmr->ifm_active |= IFM_HDX;
        // }
        if self.hw.phy.media_type != MediaType::Copper {
            unsupported!();
        }
        match self.link_speed {
            10 => ifmr.ifm_active |= IFM_10_T as i32,
            100 => ifmr.ifm_active |= IFM_100_TX as i32,
            1000 => ifmr.ifm_active |= IFM_1000_T as i32,
            _ => eprintln!("Unknown link speed"),
        }
        if self.link_duplex == FULL_DUPLEX {
            ifmr.ifm_active |= IFM_FDX as i32;
        } else {
            ifmr.ifm_active |= IFM_HDX as i32;
        }
    }
    fn attach_post(&mut self) -> AdResult {
        e1000_println!();
        /* Setup OS specific network interface */
        // 	error = em_setup_interface(ctx);
        // 	if (error != 0) {
        // 		goto err_late;
        // 	}
        try!(self.setup_interface());

        // 	em_reset(ctx);
        try!(self.reset());

        /* Initialize statistics */
        // 	em_update_stats_counters(adapter);
        // 	hw->mac.get_link_status = 1;
        // 	em_if_update_admin_status(ctx);
        // 	em_add_hw_stats(adapter);
        self.update_stats_counters();
        self.hw.mac.get_link_status = true;
        try!(self.update_admin_status());
        self.add_hw_stats();

        /* Non-AMT based hardware can now take control from firmware */
        // 	if (adapter->has_manage && !adapter->has_amt)
        // 		em_get_hw_control(adapter);
        if self.has_manage && !self.has_amt {
            self.get_hw_control();
        }
        // 	INIT_DEBUGOUT("em_if_attach_post: end");

        // println!("{:?}", self);

        Ok(())
        // 	return (error);

        // err_late:
        // 	em_release_hw_control(adapter);
        // 	em_free_pci_resources(ctx);
        // 	em_if_queues_free(ctx);
        // 	free(adapter->mta, M_DEVBUF);

        // 	return (error);
        // incomplete!();
        // Err("Fail on purpose".into())
    }
    fn stop(&mut self) -> AdResult {
        e1000_println!();
        // e1000_reset_hw(&adapter->hw);
        // if (adapter->hw.mac.type >= e1000_82544)
        // 	E1000_WRITE_REG(&adapter->hw, E1000_WUFC, 0);

        // e1000_led_off(&adapter->hw);
        // e1000_cleanup_led(&adapter->hw);
        try!(self.mac_reset_hw());
        if self.hw.mac.mac_type == MacType::Mac_82544 {
            incomplete!();
        }
        try!(self.led_off());
        try!(self.cleanup_led());
        Ok(())
    }
    fn detach(&mut self) -> AdResult {
        e1000_println!();
        // incomplete!();
        Ok(())
    }
    fn release(&mut self) {
        e1000_println!();
        // incomplete!();
        // pub memory: Option<Resource>,
        // pub ioport: Option<Resource>,
        self.ioport.take();
        self.memory.take();
    }
}

impl Drop for Adapter {
    fn drop(&mut self) {
        e1000_println!("<<<<<<<<<<<<<<<<<<<< DROP >>>>>>>>>>>>>>>>>>>>");
    }
}

pub fn is_valid_ether_addr(addr: &[u8]) -> AdResult {
    e1000_println!();
    let zero_addr = [0u8; 6];
    if addr[0] & 1 > 0 || addr == zero_addr {
        Err("Ether address is not valid".to_string())
    } else {
        Ok(())
    }
}
