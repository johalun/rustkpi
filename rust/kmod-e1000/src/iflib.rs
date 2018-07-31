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
use kernel::sys::iflib_sys::{IFCAP_TSO4, IFCAP_HWCSUM, IFCAP_LRO, IFCAP_RXCSUM, IFCAP_TXCSUM,
                             IFCAP_VLAN_HWCSUM, IFCAP_VLAN_HWFILTER, IFCAP_VLAN_HWTAGGING,
                             IFCAP_VLAN_HWTSO, IFCAP_VLAN_MTU, IFCAP_WOL, IFCAP_WOL_MAGIC,
                             IFCAP_WOL_MCAST};
use kernel::sys::mbuf_sys::{CSUM_IP_TSO, CSUM_TCP, CSUM_UDP};

use sys::e1000::{EM_DBA_ALIGN, EM_MAX_SCATTER, EM_MSIX_BAR, EM_TSO_SEG_SIZE};
use sys::e1000::*;
use sys::e1000_consts::*;
use sys::e1000::e1000_tx_desc;
use sys::e1000::e1000_rx_desc;

use sys::iflib::iflib_set_mac;
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

pub const LEM_CAPS: u32 = IFCAP_HWCSUM | IFCAP_VLAN_MTU | IFCAP_VLAN_HWTAGGING | IFCAP_VLAN_HWCSUM
    | IFCAP_WOL | IFCAP_VLAN_HWFILTER;

pub const EM_CAPS: u32 = IFCAP_HWCSUM | IFCAP_VLAN_MTU | IFCAP_VLAN_HWTAGGING | IFCAP_VLAN_HWCSUM
    | IFCAP_WOL | IFCAP_VLAN_HWFILTER | IFCAP_TSO4 | IFCAP_LRO
    | IFCAP_VLAN_HWTSO;

pub const EM_TSO_SIZE: u32 = 65535;

// #[derive(Debug)]
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
impl kernel::fmt::Debug for IfLib {
    fn fmt(&self, f: &mut kernel::fmt::Formatter) -> kernel::fmt::Result {
        write!(f, "IfLib {{ inner: {:?} }}", unsafe {
            *self.inner.as_ptr()
        })
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
    pub fn set_ifheaderlen(&mut self, len: i32) {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe {
            if_setifheaderlen(ptr, len);
        }
    }
    pub fn set_capabilitiesbit(&mut self, setbit: i32, clearbit: i32) {
        let ptr: *mut ifnet = self.inner.as_ptr();
        unsafe {
            if_setcapabilitiesbit(ptr, setbit, clearbit);
        }
    }
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
        write!(f, "IfNet {{ inner: {:?} }}", unsafe {
            *self.inner.as_ptr()
        })
    }
}

pub struct IfLibShared {
    pub inner: NonNull<if_softc_ctx>,
}
impl IfLibShared {
    pub fn setup(&mut self, mactype: MacType) -> Result<(), String> {
        e1000_println!();

        // Common
        self.isc_msix_bar = pcir_bar(EM_MSIX_BAR) as i32;
        self.isc_tx_nsegments = EM_MAX_SCATTER as i32;
        let nqs = self.num_queues(mactype);
        self.isc_nrxqsets_max = nqs;
        self.isc_ntxqsets_max = nqs;
        e1000_println!("capping queues at {}", self.isc_ntxqsets_max);

        if mactype >= MacType::IGB_MAC_MIN {
            unsupported!();
            incomplete_return!();
        } else if mactype >= MacType::EM_MAC_MIN {
            // I218/219 is here
            self.isc_txqsizes[0] = roundup2!(
                ((self.isc_ntxd[0] + 1) as u32 * kernel::mem::size_of::<e1000_tx_desc>() as u32),
                EM_DBA_ALIGN as u32
            );
            self.isc_rxqsizes[0] = roundup2!(
                ((self.isc_nrxd[0] + 1) as u32
                    * kernel::mem::size_of::<e1000_rx_desc_extended>() as u32),
                EM_DBA_ALIGN as u32
            );
            self.isc_txd_size[0] = kernel::mem::size_of::<e1000_tx_desc>() as u8;
            self.isc_rxd_size[0] = kernel::mem::size_of::<e1000_rx_desc_extended>() as u8;
            self.isc_capabilities = EM_CAPS as i32;
            self.isc_capenable = EM_CAPS as i32;
            self.isc_tx_tso_segments_max = EM_MAX_SCATTER as i32;
            self.isc_tx_tso_size_max = EM_TSO_SIZE as i32;
            self.isc_tx_tso_segsize_max = EM_TSO_SEG_SIZE as i32;
            /*
             * For EM-class devices, don't enable IFCAP_{TSO4,VLAN_HWTSO}
             * by default as we don't have workarounds for all associated
             * silicon errata.  E. g., with several MACs such as 82573E,
             * TSO only works at Gigabit speed and otherwise can cause the
             * hardware to hang (which also would be next to impossible to
             * work around given that already queued TSO-using descriptors
             * would need to be flushed and vlan(4) reconfigured at runtime
             * in case of a link speed change).  Moreover, MACs like 82579
             * still can hang at Gigabit even with all publicly documented
             * TSO workarounds implemented.  Generally, the penality of
             * these workarounds is rather high and may involve copying
             * mbuf data around so advantages of TSO lapse.  Still, TSO may
             * work for a few MACs of this class - at least when sticking
             * with Gigabit - in which case users may enable TSO manually.
             */
            self.isc_capenable &= !(IFCAP_TSO4 | IFCAP_VLAN_HWTSO) as i32;

            self.isc_tx_csum_flags = (CSUM_TCP | CSUM_UDP | CSUM_IP_TSO) as i32;
            // A struct with iflib callback functions
            self.isc_txrx = &EM_TXRX as *const if_txrx;
        } else {
            // emulated device in bhyve (82545 only)
            self.isc_txqsizes[0] = roundup2!(
                ((self.isc_ntxd[0] + 1) as u32 * kernel::mem::size_of::<e1000_tx_desc>() as u32),
                EM_DBA_ALIGN as u32
            );
            self.isc_rxqsizes[0] = roundup2!(
                ((self.isc_nrxd[0] + 1) as u32 * kernel::mem::size_of::<e1000_rx_desc>() as u32),
                EM_DBA_ALIGN as u32
            );
            self.isc_txd_size[0] = kernel::mem::size_of::<e1000_tx_desc>() as u8;
            self.isc_rxd_size[0] = kernel::mem::size_of::<e1000_rx_desc>() as u8;
            self.isc_capenable = LEM_CAPS as i32;
            self.isc_capabilities = LEM_CAPS as i32;
            if mactype < MacType::Mac_82543 {
                unsupported!();
                incomplete_return!();
            }
            self.isc_tx_csum_flags = (CSUM_TCP | CSUM_UDP) as i32;
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

pub struct IfRxdInfo {
    pub inner: NonNull<if_rxd_info>,
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
