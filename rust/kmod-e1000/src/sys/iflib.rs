/*
 * Do not use kernel::sys::iflib_sys.rs directly. Copy over used bits to this
 * file instead. This is because we need to modify bindgen generated bindings.
 */

use kernel::prelude::v1::*;
use kernel::sys::raw::*;

use kernel::sys::kernel_sys::caddr_t;

// Re-export unmodified
pub use kernel::sys::iflib_sys::bool_;
pub use kernel::sys::iflib_sys::qidx_t;
pub use kernel::sys::iflib_sys::if_rxd_update;
pub use kernel::sys::iflib_sys::if_rxd_info;
pub use kernel::sys::iflib_sys::if_pkt_info;
pub use kernel::sys::iflib_sys::iflib_ctx;
pub use kernel::sys::iflib_sys::ifmedia;
pub use kernel::sys::iflib_sys::ifmediareq;
pub use kernel::sys::iflib_sys::device;
pub use kernel::sys::iflib_sys::ifnet;
pub use kernel::sys::iflib_sys::if_shared_ctx;

use e1000_osdep::Resource;

extern "C" {
    pub fn iflib_set_mac(ctx: *mut iflib_ctx, mac: *const u8);
}
extern "C" {
    pub fn iflib_get_softc(ctx: *mut iflib_ctx) -> *mut c_void;
}
extern "C" {
    pub fn iflib_get_dev(ctx: *mut iflib_ctx) -> *mut device;
}
extern "C" {
    pub fn iflib_get_ifp(ctx: *mut iflib_ctx) -> *mut ifnet;
}
extern "C" {
    pub fn iflib_get_media(ctx: *mut iflib_ctx) -> *mut ifmedia;
}
extern "C" {
    pub fn iflib_get_softc_ctx(ctx: *mut iflib_ctx) -> *mut if_softc_ctx;
}
extern "C" {
    pub fn iflib_get_sctx(ctx: *mut iflib_ctx) -> *mut if_shared_ctx;
}

extern "C" {
    pub fn if_get_counter_default(arg1: *mut ifnet, arg2: ift_counter) -> u64;
}
extern "C" {
    pub fn if_getlladdr(ifp: *mut ifnet) -> caddr_t;
}
extern "C" {
    pub fn if_multiaddr_array(
        ifp: *mut ifnet,
        mta: *mut ::kernel::sys::raw::c_void,
        cnt: *mut u32,
        max: u32,
    ) -> i32;
}

#[repr(C)]
#[derive(Debug, Copy)]
pub struct if_softc_ctx {
    pub isc_vectors: ::kernel::sys::raw::c_int,
    pub isc_nrxqsets: ::kernel::sys::raw::c_int,
    pub isc_ntxqsets: ::kernel::sys::raw::c_int,
    pub isc_min_tx_latency: u8,
    pub isc_rx_mvec_enable: u8,
    pub isc_txrx_budget_bytes_max: u32,
    pub isc_msix_bar: ::kernel::sys::raw::c_int,
    pub isc_tx_nsegments: ::kernel::sys::raw::c_int,
    pub isc_ntxd: [::kernel::sys::raw::c_int; 8usize],
    pub isc_nrxd: [::kernel::sys::raw::c_int; 8usize],
    pub isc_txqsizes: [u32; 8usize],
    pub isc_rxqsizes: [u32; 8usize],
    pub isc_txd_size: [u8; 8usize],
    pub isc_rxd_size: [u8; 8usize],
    pub isc_tx_tso_segments_max: ::kernel::sys::raw::c_int,
    pub isc_tx_tso_size_max: ::kernel::sys::raw::c_int,
    pub isc_tx_tso_segsize_max: ::kernel::sys::raw::c_int,
    pub isc_tx_csum_flags: ::kernel::sys::raw::c_int,
    pub isc_capenable: ::kernel::sys::raw::c_int,
    pub isc_rss_table_size: ::kernel::sys::raw::c_int,
    pub isc_rss_table_mask: ::kernel::sys::raw::c_int,
    pub isc_nrxqsets_max: ::kernel::sys::raw::c_int,
    pub isc_ntxqsets_max: ::kernel::sys::raw::c_int,
    pub isc_tx_qdepth: u32,
    pub isc_intr: iflib_intr_mode_t,
    pub isc_max_frame_size: u16,
    pub isc_min_frame_size: u16,
    pub isc_pause_frames: u32,
    pub isc_vendor_info: pci_vendor_info,
    pub isc_disable_msix: ::kernel::sys::raw::c_int,
    pub isc_txrx: *const if_txrx,
}
impl Clone for if_softc_ctx {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for if_softc_ctx {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct if_txrx {
    pub ift_txd_encap: ::core::option::Option<
        unsafe extern "C" fn(arg1: *mut ::kernel::sys::raw::c_void, arg2: *mut if_pkt_info)
            -> ::kernel::sys::raw::c_int,
    >,
    pub ift_txd_flush: ::core::option::Option<
        unsafe extern "C" fn(arg1: *mut ::kernel::sys::raw::c_void, arg2: u16, pidx: qidx_t),
    >,
    pub ift_txd_credits_update: ::core::option::Option<
        unsafe extern "C" fn(arg1: *mut ::kernel::sys::raw::c_void, qsidx: u16, clear: bool_)
            -> ::kernel::sys::raw::c_int,
    >,
    pub ift_rxd_available: ::core::option::Option<
        unsafe extern "C" fn(
            arg1: *mut ::kernel::sys::raw::c_void,
            qsidx: u16,
            pidx: qidx_t,
            budget: qidx_t,
        ) -> ::kernel::sys::raw::c_int,
    >,
    pub ift_rxd_pkt_get: ::core::option::Option<
        unsafe extern "C" fn(arg1: *mut ::kernel::sys::raw::c_void, ri: *mut if_rxd_info)
            -> ::kernel::sys::raw::c_int,
    >,
    pub ift_rxd_refill: ::core::option::Option<
        unsafe extern "C" fn(arg1: *mut ::kernel::sys::raw::c_void, iru: *mut if_rxd_update),
    >,
    pub ift_rxd_flush: ::core::option::Option<
        unsafe extern "C" fn(
            arg1: *mut ::kernel::sys::raw::c_void,
            qsidx: u16,
            flidx: u8,
            pidx: qidx_t,
        ),
    >,
    pub ift_legacy_intr: ::core::option::Option<
        unsafe extern "C" fn(arg1: *mut ::kernel::sys::raw::c_void) -> ::kernel::sys::raw::c_int,
    >,
}
// impl Clone for if_txrx {
//     fn clone(&self) -> Self {
//         *self
//     }
// }
// impl Default for if_txrx {
//     fn default() -> Self {
//         unsafe { ::core::mem::zeroed() }
//     }
// }

#[derive(Debug)]
pub struct IfIrq {
    pub ii_res: Resource,
    pub ii_rid: ::kernel::sys::raw::c_int,
    pub ii_tag: *mut ::kernel::sys::raw::c_void,
}
// impl Clone for IfIrq {
//     fn clone(&self) -> Self {
//         *self
//     }
// }
impl Default for IfIrq {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Debug, Copy)]
pub struct pci_vendor_info {
    pub pvi_vendor_id: u32,
    pub pvi_device_id: u32,
    pub pvi_subvendor_id: u32,
    pub pvi_subdevice_id: u32,
    pub pvi_rev_id: u32,
    pub pvi_class_mask: u32,
    pub pvi_name: caddr_t,
}
impl Clone for pci_vendor_info {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for pci_vendor_info {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum iflib_intr_mode_t {
    IFLIB_INTR_LEGACY = 0,
    IFLIB_INTR_MSI = 1,
    IFLIB_INTR_MSIX = 2,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IftCounter {
    IPACKETS = 0,
    IERRORS = 1,
    OPACKETS = 2,
    OERRORS = 3,
    COLLISIONS = 4,
    IBYTES = 5,
    OBYTES = 6,
    IMCASTS = 7,
    OMCASTS = 8,
    IQDROPS = 9,
    OQDROPS = 10,
    NOPROTO = 11,
    IFCOUNTERS = 12,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ift_counter {
    IFCOUNTER_IPACKETS = 0,
    IFCOUNTER_IERRORS = 1,
    IFCOUNTER_OPACKETS = 2,
    IFCOUNTER_OERRORS = 3,
    IFCOUNTER_COLLISIONS = 4,
    IFCOUNTER_IBYTES = 5,
    IFCOUNTER_OBYTES = 6,
    IFCOUNTER_IMCASTS = 7,
    IFCOUNTER_OMCASTS = 8,
    IFCOUNTER_IQDROPS = 9,
    IFCOUNTER_OQDROPS = 10,
    IFCOUNTER_NOPROTO = 11,
    IFCOUNTERS = 12,
}
impl ift_counter {
    pub fn from(cnt: IftCounter) -> ift_counter {
        match cnt {
            IftCounter::IPACKETS => ift_counter::IFCOUNTER_IPACKETS,
            IftCounter::IERRORS => ift_counter::IFCOUNTER_IERRORS,
            IftCounter::OPACKETS => ift_counter::IFCOUNTER_OPACKETS,
            IftCounter::OERRORS => ift_counter::IFCOUNTER_OERRORS,
            IftCounter::COLLISIONS => ift_counter::IFCOUNTER_COLLISIONS,
            IftCounter::IBYTES => ift_counter::IFCOUNTER_IBYTES,
            IftCounter::OBYTES => ift_counter::IFCOUNTER_OBYTES,
            IftCounter::IMCASTS => ift_counter::IFCOUNTER_IMCASTS,
            IftCounter::OMCASTS => ift_counter::IFCOUNTER_OMCASTS,
            IftCounter::IQDROPS => ift_counter::IFCOUNTER_IQDROPS,
            IftCounter::OQDROPS => ift_counter::IFCOUNTER_OQDROPS,
            IftCounter::NOPROTO => ift_counter::IFCOUNTER_NOPROTO,
            IftCounter::IFCOUNTERS => ift_counter::IFCOUNTERS,
        }
    }
}
