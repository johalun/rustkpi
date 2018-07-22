
use kernel;
use kernel::ptr::NonNull;
use kernel::sys::raw::*;
use kernel::prelude::v1::*;

use kernel::sys::iflib_sys::ifnet;
use kernel::sys::iflib_sys::ifmediareq;
use kernel::sys::iflib_sys::if_sethwtsomax;
use kernel::sys::iflib_sys::if_sethwtsomaxsegcount;
use kernel::sys::iflib_sys::if_sethwtsomaxsegsize;
use kernel::sys::iflib_sys::if_setsendqlen;
use kernel::sys::iflib_sys::if_getmtu;
use kernel::sys::iflib_sys::if_setsendqready;
use kernel::sys::iflib_sys::if_setifheaderlen;
use kernel::sys::iflib_sys::if_setcapenablebit;
use kernel::sys::iflib_sys::if_setcapabilitiesbit;
use kernel::sys::iflib_sys::if_setbaudrate;
use kernel::sys::iflib_sys::if_getflags;
use kernel::sys::iflib_sys::if_getcapenable;
use kernel::sys::iflib_sys::if_multiaddr_count;

use sys::iflib::if_multiaddr_array;
use sys::iflib::if_get_counter_default;
use sys::iflib::if_getlladdr;

use kernel::sys::iflib_sys::iflib_get_softc;
use kernel::sys::iflib_sys::iflib_get_ifp;
use kernel::sys::iflib_sys::iflib_link_state_change;
use kernel::sys::iflib_sys::iflib_admin_intr_deferred;
use kernel::sys::iflib_sys::if_pkt_info;
use kernel::sys::iflib_sys::if_rxd_info;
use kernel::sys::iflib_sys::if_rxd_frag;
use kernel::sys::iflib_sys::if_rxd_update;
use kernel::sys::iflib_sys::qidx_t;
use kernel::sys::iflib_sys::bool_;
use kernel::sys::iflib_sys::iflib_ctx;
use kernel::sys::iflib_sys::ifmedia;
use kernel::sys::iflib_sys::ifmedia_add;
use kernel::sys::iflib_sys::ifmedia_set;
use kernel::sys::iflib_sys::ether_vlan_header;
use kernel::sys::iflib_sys::ETHER_ADDR_LEN;
use kernel::sys::iflib_sys::{IFCAP_TSO4, IFCAP_TXCSUM, IFCAP_LRO, IFCAP_RXCSUM,
                             IFCAP_VLAN_HWFILTER, IFCAP_WOL_MAGIC, IFCAP_WOL_MCAST, IFCAP_WOL,
                             IFCAP_VLAN_HWTSO, IFCAP_HWCSUM, IFCAP_VLAN_HWTAGGING,
                             IFCAP_VLAN_HWCSUM, IFCAP_VLAN_MTU};
use kernel::sys::mbuf_sys::{CSUM_TCP, CSUM_UDP, CSUM_IP_TSO};

use sys::e1000::{EM_MSIX_BAR, EM_MAX_SCATTER, EM_TSO_SEG_SIZE, EM_DBA_ALIGN};
use sys::e1000::*;
use sys::e1000_consts::*;
use sys::e1000::e1000_tx_desc;
use sys::e1000::e1000_rx_desc;

use sys::iflib::iflib_set_mac;
// use sys::iflib::if_pkt_info;
use sys::iflib::if_txrx;
use sys::iflib::if_softc_ctx;
pub use sys::iflib::IftCounter;
pub use sys::iflib::ift_counter;

use LEM_TXRX;
use EM_TXRX;

use adapter::Adapter;

pub const PCIR_BARS: u32 = 0x10;
pub const fn pcir_bar(x: u32) -> u32 {
    (PCIR_BARS + (x * 4))
}

pub const EM_CAPS: u32 = IFCAP_TSO4 | IFCAP_TXCSUM | IFCAP_LRO | IFCAP_RXCSUM |
    IFCAP_VLAN_HWFILTER | IFCAP_WOL_MAGIC | IFCAP_WOL_MCAST |
    IFCAP_WOL | IFCAP_VLAN_HWTSO | IFCAP_HWCSUM | IFCAP_VLAN_HWTAGGING |
    IFCAP_VLAN_HWCSUM | IFCAP_VLAN_MTU;


// #define EM_TSO_SIZE		(65535 + sizeof(struct ether_vlan_header))
pub const EM_TSO_SIZE: u32 = (65535 + kernel::mem::size_of::<ether_vlan_header>() as u32);




#[derive(Debug)]
pub struct IfLib {
    pub inner: NonNull<iflib_ctx>, // iflib internal struct
}
impl IfLib {
    pub fn set_mac(&mut self, addr: &[u8]) {
        let ctx: *mut iflib_ctx = self.inner.as_ptr();
        unsafe {
            iflib_set_mac(ctx, addr.as_ptr());
        }
    }
    pub fn link_state_change(&mut self, state: i32, baudrate: u64) {
        let ctx: *mut iflib_ctx = self.inner.as_ptr();
        unsafe {
            iflib_link_state_change(ctx, state, baudrate);
        }
    }
    pub fn admin_intr_deferred(&mut self) {
        let ctx: *mut iflib_ctx = self.inner.as_ptr();
        unsafe {
            iflib_admin_intr_deferred(ctx);
        }
    }
}


pub struct IfNet {
    pub inner: NonNull<ifnet>, // if internal struct
}
impl IfNet {
    pub fn set_hwtsomax(&mut self, v: u32) {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe {
            if_sethwtsomax(ptr, v);
        }
    }
    pub fn set_hwtsomax_segcount(&mut self, v: u32) {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe {
            if_sethwtsomaxsegcount(ptr, v);
        }
    }
    pub fn set_hwtsomax_segsize(&mut self, v: u32) {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe {
            if_sethwtsomaxsegsize(ptr, v);
        }
    }
    pub fn set_sendqlen(&mut self, v: i32) {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe {
            if_setsendqlen(ptr, v);
        }
    }
    pub fn set_sendqready(&mut self) {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe {
            if_setsendqready(ptr);
        }
    }
    // if_setifheaderlen(ifp, sizeof(struct ether_vlan_header));
    pub fn set_ifheaderlen(&mut self, len: i32) {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe {
            if_setifheaderlen(ptr, len);
        }
    }
    // if_setcapabilitiesbit(ifp, cap, 0);
    pub fn set_capabilitiesbit(&mut self, setbit: i32, clearbit: i32) {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe {
            if_setcapabilitiesbit(ptr, setbit, clearbit);
        }
    }
    // if_setcapenablebit(ifp, cap, 0);
    pub fn set_capenablebit(&mut self, setcap: i32, clearcap: i32) -> i32 {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe { if_setcapenablebit(ptr, setcap, clearcap) }
    }
    pub fn set_baudrate(&mut self, rate: u64) {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe {
            if_setbaudrate(ptr, rate);
        }
    }
    pub fn counter_default(&mut self, cnt: IftCounter) -> u64 {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe { if_get_counter_default(ptr, ift_counter::from(cnt)) }
    }
    pub fn lladdr(&self) -> [u8; ETHER_ADDR_LEN as usize] {
        let ptr: *mut ifnet = self.inner.as_ptr();
        let addr_ptr: *mut u8 = unsafe { if_getlladdr(ptr) } as *mut u8;
        let addr: &[u8] =
            unsafe { kernel::slice::from_raw_parts(addr_ptr, ETHER_ADDR_LEN as usize) };
        let mut ret = [0u8; ETHER_ADDR_LEN as usize];
        ret.copy_from_slice(addr);
        ret
    }
    pub fn multiaddr_array(&mut self, mta: *mut u8, mcnt: &mut u32, max: u32) {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe {
            if_multiaddr_array(ptr, mta as *mut kernel::sys::c_void, mcnt, max);
        }
    }
    pub fn mtu(&mut self) -> i32 {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe { if_getmtu(ptr) }
    }
    pub fn capenable(&mut self) -> u32 {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe { if_getcapenable(ptr) as u32 }
    }
    pub fn flags(&mut self) -> u32 {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe { if_getflags(ptr) as u32 }
    }
    pub fn multiaddr_count(&mut self, max: u32) -> u32 {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe { if_multiaddr_count(ptr, max as i32) as u32 }
    }
}
impl kernel::ops::Deref for IfNet {
    type Target = ifnet;
    fn deref(&self) -> &ifnet {
        unsafe { self.inner.as_ref() }
    }
}
impl kernel::ops::DerefMut for IfNet {
    fn deref_mut(&mut self) -> &mut ifnet {
        unsafe { self.inner.as_mut() }
    }
}
impl kernel::fmt::Debug for IfNet {
    fn fmt(&self, f: &mut kernel::fmt::Formatter) -> kernel::fmt::Result {
        write!(
            f,
            "IfNet {{ inner: {:?} }}",
            unsafe { *self.inner.as_ptr() }
        )
    }
}




pub struct IfLibShared {
    pub inner: NonNull<if_softc_ctx>,
    // pub struct if_softc_ctx {
    //     pub isc_vectors: ::kernel::sys::raw::c_int,
    //     pub isc_nrxqsets: ::kernel::sys::raw::c_int,
    //     pub isc_ntxqsets: ::kernel::sys::raw::c_int,
    //     pub isc_msix_bar: ::kernel::sys::raw::c_int,
    //     pub isc_tx_nsegments: ::kernel::sys::raw::c_int,
    //     pub isc_ntxd: [::kernel::sys::raw::c_int; 8usize],
    //     pub isc_nrxd: [::kernel::sys::raw::c_int; 8usize],
    //     pub isc_txqsizes: [u32; 8usize],
    //     pub isc_rxqsizes: [u32; 8usize],
    //     pub isc_txd_size: [u8; 8usize],
    //     pub isc_rxd_size: [u8; 8usize],
    //     pub isc_tx_tso_segments_max: ::kernel::sys::raw::c_int,
    //     pub isc_tx_tso_size_max: ::kernel::sys::raw::c_int,
    //     pub isc_tx_tso_segsize_max: ::kernel::sys::raw::c_int,
    //     pub isc_tx_csum_flags: ::kernel::sys::raw::c_int,
    //     pub isc_capenable: ::kernel::sys::raw::c_int,
    //     pub isc_rss_table_size: ::kernel::sys::raw::c_int,
    //     pub isc_rss_table_mask: ::kernel::sys::raw::c_int,
    //     pub isc_nrxqsets_max: ::kernel::sys::raw::c_int,
    //     pub isc_ntxqsets_max: ::kernel::sys::raw::c_int,
    //     pub isc_intr: iflib_intr_mode_t,
    //     pub isc_max_frame_size: u16,
    //     pub isc_min_frame_size: u16,
    //     pub isc_pause_frames: u32,
    //     pub isc_vendor_info: pci_vendor_info_t,
    //     pub isc_disable_msix: ::kernel::sys::raw::c_int,
    //     pub isc_txrx: if_txrx_t,
    // }
}
impl IfLibShared {
    pub fn setup(&mut self, mactype: MacType) -> Result<(), String> {
        e1000_println!();

        // Common
        self.isc_msix_bar = pcir_bar(EM_MSIX_BAR) as i32;
        self.isc_tx_nsegments = EM_MAX_SCATTER as i32;
        self.isc_tx_tso_segments_max = self.isc_tx_nsegments;
        self.isc_tx_tso_size_max = EM_TSO_SIZE as i32;
        self.isc_tx_tso_segsize_max = EM_TSO_SEG_SIZE as i32;
        let nqs = self.num_queues(mactype);
        self.isc_nrxqsets_max = nqs;
        self.isc_ntxqsets_max = nqs;
        e1000_println!("capping queues at {}", self.isc_ntxqsets_max);
        self.isc_tx_csum_flags = (CSUM_TCP | CSUM_UDP | CSUM_IP_TSO) as i32;

        if mactype >= MacType::IGB_MAC_MIN {
            unsupported!();
            incomplete_return!();
        } else if mactype >= MacType::EM_MAC_MIN {
            // Real hardware
            self.isc_txqsizes[0] = roundup2!(
                ((self.isc_ntxd[0] + 1) as u32 *
                     kernel::mem::size_of::<e1000_tx_desc>() as u32),
                EM_DBA_ALIGN as u32
            );
            self.isc_rxqsizes[0] = roundup2!(
                ((self.isc_nrxd[0] + 1) as u32 *
                     kernel::mem::size_of::<e1000_rx_desc_extended>() as u32),
                EM_DBA_ALIGN as u32
            );
            self.isc_txd_size[0] = kernel::mem::size_of::<e1000_tx_desc>() as u8;
            self.isc_rxd_size[0] = kernel::mem::size_of::<e1000_rx_desc_extended>() as u8;
            self.isc_capenable = EM_CAPS as i32;
            self.isc_tx_csum_flags = (CSUM_TCP | CSUM_UDP | CSUM_IP_TSO) as i32;
            // A struct with iflib callback functions
            self.isc_txrx = &EM_TXRX as *const if_txrx;
        } else {
            // bhyve emulation (82545 only)
            self.isc_txqsizes[0] = roundup2!(
                ((self.isc_ntxd[0] + 1) as u32 *
                     kernel::mem::size_of::<e1000_tx_desc>() as u32),
                EM_DBA_ALIGN as u32
            );
            self.isc_rxqsizes[0] = roundup2!(
                ((self.isc_nrxd[0] + 1) as u32 *
                     kernel::mem::size_of::<e1000_rx_desc>() as u32),
                EM_DBA_ALIGN as u32
            );
            self.isc_txd_size[0] = kernel::mem::size_of::<e1000_tx_desc>() as u8;
            self.isc_rxd_size[0] = kernel::mem::size_of::<e1000_rx_desc>() as u8;
            self.isc_capenable = EM_CAPS as i32;
            if mactype < MacType::Mac_82543 {
                unsupported!();
                incomplete_return!();
            }
            self.isc_tx_csum_flags = (CSUM_TCP | CSUM_UDP | CSUM_IP_TSO) as i32;
            self.isc_msix_bar = 0;

            // A struct with iflib callback functions
            self.isc_txrx = &LEM_TXRX as *const if_txrx;
        }
        Ok(())
    }
    pub fn num_queues(&self, mactype: MacType) -> i32 {
        e1000_println!();
        /* Sanity check based on HW */
        match mactype {
            MacType::Mac_82545 => 1,
            MacType::Mac_pch_lpt => 1,
            MacType::Mac_pch_spt => 1,
            _ => {
                unsupported!();
                1
            }
        }
    }
    pub fn tx_process_limit(&self) -> i32 {
        e1000_println!();
        self.isc_ntxd[0]
    }
    pub fn rx_process_limit(&self) -> i32 {
        e1000_println!();
        self.isc_nrxd[0]
    }
}
impl kernel::ops::Deref for IfLibShared {
    type Target = if_softc_ctx;
    fn deref(&self) -> &if_softc_ctx {
        unsafe { self.inner.as_ref() }
    }
}
impl kernel::ops::DerefMut for IfLibShared {
    fn deref_mut(&mut self) -> &mut if_softc_ctx {
        unsafe { self.inner.as_mut() }
    }
}
impl kernel::fmt::Debug for IfLibShared {
    fn fmt(&self, f: &mut kernel::fmt::Formatter) -> kernel::fmt::Result {
        write!(f, "IfLibShared {{ inner: {:?} }}", unsafe {
            *self.inner.as_ptr()
        })
    }
}


#[derive(Debug)]
pub struct IfMedia {
    pub inner: NonNull<ifmedia>, // iflib internal struct
}
impl IfMedia {
    // pub fn ifmedia_add(ifm: *mut ifmedia, mword: ::kernel::sys::raw::c_int,
    //                    data: ::kernel::sys::raw::c_int,
    //                    aux: *mut ::kernel::sys::raw::c_void);
    pub fn add(&mut self, mword: i32, data: i32, aux: *mut c_void) {
        let ptr: *mut ifmedia = self.inner.as_ptr();
        unsafe {
            ifmedia_add(ptr, mword, data, aux);
        }
    }
    pub fn set(&mut self, mword: i32) {
        let ptr: *mut ifmedia = self.inner.as_ptr();
        unsafe {
            ifmedia_set(ptr, mword);
        }
    }
}

pub struct IfMediaReq {
    pub inner: NonNull<ifmediareq>,
}
impl kernel::ops::Deref for IfMediaReq {
    type Target = ifmediareq;
    fn deref(&self) -> &ifmediareq {
        unsafe { self.inner.as_ref() }
    }
}
impl kernel::ops::DerefMut for IfMediaReq {
    fn deref_mut(&mut self) -> &mut ifmediareq {
        unsafe { self.inner.as_mut() }
    }
}
impl kernel::fmt::Debug for IfMediaReq {
    fn fmt(&self, f: &mut kernel::fmt::Formatter) -> kernel::fmt::Result {
        write!(f, "IfMediaReq {{ inner: {:?} }}", unsafe {
            *self.inner.as_ptr()
        })
    }
}

pub struct IfRxdUpdate {
    pub inner: NonNull<if_rxd_update>,
// pub struct if_rxd_update {
//     pub iru_paddrs: *mut u64,
//     pub iru_vaddrs: *mut caddr_t,
//     pub iru_idxs: *mut qidx_t,
//     pub iru_pidx: qidx_t,
//     pub iru_qsidx: u16,
//     pub iru_count: u16,
//     pub iru_buf_size: u16,
//     pub iru_flidx: u8,
// }
}
impl kernel::ops::Deref for IfRxdUpdate {
    type Target = if_rxd_update;
    fn deref(&self) -> &if_rxd_update {
        unsafe { self.inner.as_ref() }
    }
}
impl kernel::ops::DerefMut for IfRxdUpdate {
    fn deref_mut(&mut self) -> &mut if_rxd_update {
        unsafe { self.inner.as_mut() }
    }
}
impl kernel::fmt::Debug for IfRxdUpdate {
    fn fmt(&self, f: &mut kernel::fmt::Formatter) -> kernel::fmt::Result {
        write!(f, "IfRxdUpdate {{ inner: {:?} }}", unsafe {
            *self.inner.as_ptr()
        })
    }
}

pub struct IfPacketInfo {
    pub inner: NonNull<if_pkt_info>,
    // pub ipi_segs: *mut bus_dma_segment_t,
    // pub ipi_len: u32,
    // pub ipi_qsidx: u16,
    // pub ipi_nsegs: qidx_t,
    // pub ipi_ndescs: qidx_t,
    // pub ipi_flags: u16,
    // pub ipi_pidx: qidx_t,
    // pub ipi_new_pidx: qidx_t,
    // pub ipi_ehdrlen: u8,
    // pub ipi_ip_hlen: u8,
    // pub ipi_tcp_hlen: u8,
    // pub ipi_ipproto: u8,
    // pub ipi_csum_flags: u32,
    // pub ipi_tso_segsz: u16,
    // pub ipi_vtag: u16,
    // pub ipi_etype: u16,
    // pub ipi_tcp_hflags: u8,
    // pub ipi_mflags: u8,
    // pub ipi_tcp_seq: u32,
    // pub ipi_tcp_sum: u32,
}
impl kernel::ops::Deref for IfPacketInfo {
    type Target = if_pkt_info;
    fn deref(&self) -> &if_pkt_info {
        unsafe { self.inner.as_ref() }
    }
}
impl kernel::ops::DerefMut for IfPacketInfo {
    fn deref_mut(&mut self) -> &mut if_pkt_info {
        unsafe { self.inner.as_mut() }
    }
}
impl kernel::fmt::Debug for IfPacketInfo {
    fn fmt(&self, f: &mut kernel::fmt::Formatter) -> kernel::fmt::Result {
        write!(f, "IfPacketInfo {{ inner: {:?} }}", unsafe {
            *self.inner.as_ptr()
        })
    }
}

// #[derive(Debug, Default)]
// pub struct if_rxd_frag {
//     pub irf_flid: u8,
//     pub irf_idx: u16,
//     pub irf_len: u16,
// }

pub struct IfRxdInfo {
    pub inner: NonNull<if_rxd_info>,
    // pub iri_qsidx: u16,
    // pub iri_vtag: u16,
    // pub iri_len: u16,
    // pub iri_cidx: qidx_t,
    // pub iri_ifp: *mut ifnet,
    // pub iri_frags: if_rxd_frag_t, (if_rxd_frag_t = *mut if_rxd_frag)
    // pub iri_flowid: u32,
    // pub iri_csum_flags: u32,
    // pub iri_csum_data: u32,
    // pub iri_flags: u8,
    // pub iri_nfrags: u8,
    // pub iri_rsstype: u8,
    // pub iri_pad: u8,
}
impl IfRxdInfo {
    pub fn frags_slice(&mut self, len: usize) -> &mut [if_rxd_frag] {
        unsafe { kernel::slice::from_raw_parts_mut(self.iri_frags, len) }
    }
}
impl kernel::ops::Deref for IfRxdInfo {
    type Target = if_rxd_info;
    fn deref(&self) -> &if_rxd_info {
        unsafe { self.inner.as_ref() }
    }
}
impl kernel::ops::DerefMut for IfRxdInfo {
    fn deref_mut(&mut self) -> &mut if_rxd_info {
        unsafe { self.inner.as_mut() }
    }
}
impl kernel::fmt::Debug for IfRxdInfo {
    fn fmt(&self, f: &mut kernel::fmt::Formatter) -> kernel::fmt::Result {
        write!(f, "IfRxdInfo {{ inner: {:?} }}", unsafe {
            *self.inner.as_ptr()
        })
    }
}



// typedef struct if_txrx {
// 	int (*ift_txd_encap) (void *, if_pkt_info_t);
// 	void (*ift_txd_flush) (void *, uint16_t, qidx_t pidx);
// 	int (*ift_txd_credits_update) (void *, uint16_t qsidx, bool clear);

// 	int (*ift_rxd_available) (void *, uint16_t qsidx, qidx_t pidx, qidx_t budget);
// 	int (*ift_rxd_pkt_get) (void *, if_rxd_info_t ri);
// 	void (*ift_rxd_refill) (void * , if_rxd_update_t iru);
// 	void (*ift_rxd_flush) (void *, uint16_t qsidx, uint8_t flidx, qidx_t pidx);
// 	int (*ift_legacy_intr) (void *);
// } *if_txrx_t;

// struct if_txrx lem_txrx = {
//     em_isc_txd_encap,
//     em_isc_txd_flush,
//     em_isc_txd_credits_update,
//     lem_isc_rxd_available,
//     lem_isc_rxd_pkt_get,
//     lem_isc_rxd_refill,
//     em_isc_rxd_flush,
//     em_intr
// };
