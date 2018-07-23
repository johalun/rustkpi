use kernel;
use kernel::sys::raw::*;
use kernel::prelude::v1::*;
use kernel::ptr::NonNull;

use kernel::sys::kernel_sys::caddr_t;

use sys::iflib::iflib_ctx;
use sys::iflib::if_softc_ctx;
use sys::iflib::ifmedia;
use sys::iflib::iflib_get_dev;
use sys::iflib::iflib_get_softc;
use sys::iflib::iflib_get_softc_ctx;
use sys::iflib::iflib_get_media;
use sys::iflib::device;

use iflib::*;
use e1000_osdep::*;
use hw::*;
use consts::*;
use adapter::*;

pub trait Ifdi {
    fn init_pre(
        &mut self,
        dev: PciDevice,
        iflib: IfLib,
        iflib_shared: IfLibShared,
        media: IfMedia,
        ifnet: IfNet,
    ) -> Result<(), String>;
    fn init(&mut self) -> Result<(), String>;
    fn attach_pre(&mut self) -> Result<(), String>;
    fn tx_queues_alloc(
        &mut self,
        vaddrs: *mut caddr_t,
        paddrs: *mut u64,
        ntxqs: usize,
        ntxqsets: usize,
    ) -> Result<(), String>;
    fn rx_queues_alloc(
        &mut self,
        vaddrs: *mut caddr_t,
        paddrs: *mut u64,
        nrxqs: usize,
        nrxqsets: usize,
    ) -> Result<(), String>;
    fn enable_intr(&mut self);
    fn disable_intr(&mut self);
    fn timer(&mut self, qid: u16);
    fn get_counter(&mut self, cnt: IftCounter) -> u64;
    fn media_status(&mut self, ifmr: &mut IfMediaReq);
    fn attach_post(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn detach(&mut self) -> Result<(), String>;
    fn release(&mut self);
}

pub trait IfTxRx {
    fn em_txd_encap(&mut self, pi: &mut IfPacketInfo) -> i32;
    fn em_txd_flush(&mut self, txqid: u16, pidx: u16);
    fn em_txd_credits_update(&mut self, txqid: u16, clear: bool) -> i32;
    fn em_rxd_available(&mut self, rxqid: u16, idx: u16, budget: u16) -> i32;
    fn lem_rxd_available(&mut self, rxqid: u16, idx: u16, budget: u16) -> i32;
    fn em_rxd_pkt_get(&mut self, ri: &mut IfRxdInfo) -> i32;
    fn lem_rxd_pkt_get(&mut self, ri: &mut IfRxdInfo) -> i32;
    fn em_rxd_refill(&mut self, iru: &mut IfRxdUpdate);
    fn lem_rxd_refill(&mut self, iru: &mut IfRxdUpdate);
    fn em_rxd_flush(&mut self, rxqid: u16, flid: u8, pidx: u16);
    fn em_intr(&mut self) -> i32;
}
