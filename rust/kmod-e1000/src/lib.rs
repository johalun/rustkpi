#![feature(iterator_step_by, raw, shared, const_fn, plugin, used, global_asm, rustc_private,
           unique, untagged_unions, ptr_internals)]
#![feature(core_intrinsics)]
#![feature(type_ascription)]
#![feature(nll)]
#![allow(dead_code, unused_variables, safe_packed_borrows, unused_imports)]
#![no_std]

// #![feature(trace_macros)]
// trace_macros!(true);

#[macro_use]
extern crate kernel;

#[macro_use]
mod macros;

#[macro_use]
mod macros_freebsd;

mod consts;
pub use consts::*;

mod hw;
use hw::*;

mod adapter;
use adapter::*;

mod iflib;
use iflib::*;

mod bridge;
use bridge::*;

mod e1000_osdep;
use e1000_osdep::*;

mod e1000_txrx;
use e1000_txrx::*;

mod e1000_mac;
mod e1000_82540;
mod e1000_82541;
mod e1000_82542;
mod e1000_82543;
mod e1000_ich8lan;
mod e1000_phy;
mod e1000_nvm;
mod e1000_mbx;
mod e1000_manage;

#[allow(non_snake_case)]
mod e1000_regs;

#[allow(dead_code, improper_ctypes, non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod sys;

// kernel functions and structures
use kernel::ptr::NonNull;
use kernel::sys::raw::*;
use kernel::prelude::v1::*;
use kernel::raw::TraitObject;

use kernel::sys::kernel_sys::caddr_t;

// iflib functions
use sys::iflib::iflib_get_dev;
use sys::iflib::iflib_get_media;
use sys::iflib::iflib_get_softc;
use sys::iflib::iflib_get_ifp;
use sys::iflib::iflib_get_softc_ctx;

// iflib structures
use sys::iflib::iflib_ctx;
use sys::iflib::ift_counter;
use sys::iflib::ifmedia;
use sys::iflib::ifnet;
use sys::iflib::ifmediareq;
use sys::iflib::device;
use sys::iflib::if_softc_ctx;

// e1000 functions
// use sys::e1000::e1000_phy_hw_reset;

// e1000 constants and enums
// use sys::e1000::e1000_mac_type;
// use sys::e1000_consts::*;

// e1000 structures
// use sys::e1000::adapter as e1000_adapter;
// use sys::e1000::e1000_hw;
// use sys::e1000::e1000_mac_info;

pub static DEBUG_PRINT: bool = false;
pub static DEBUG_PHY_PRINT: bool = false;
pub static DEBUG_MAC_PRINT: bool = false;
pub static DEBUG_VERBOSE_PRINT: bool = false;

#[inline]
pub fn printf(msg: &str) {
    // log(msg);
    unsafe {
        kernel::sys::systm_sys::uprintf(msg.as_ptr() as *const i8);
    }
}

#[inline]
pub fn log(msg: &str) {
    unsafe {
        kernel::sys::systm_sys::log(0, msg.as_ptr() as *const i8);
    }
}

#[no_mangle]
pub extern "C" fn rust_em_adapter_size() -> usize {
    let size = kernel::mem::size_of::<Adapter>();
    e1000_println!("size = {}", size);
    size
}

#[no_mangle]
pub extern "C" fn rust_em_if_attach_pre(iflib_ptr: *mut iflib_ctx) -> i32 {
    // printf("rust_em_if_attach_pre\n\0");
    e1000_println!();

    // adapter initialized to 0's here
    let adapter_ptr: *mut c_void = unsafe { iflib_get_softc(iflib_ptr) };
    e1000_println!("adapter ptr: {:?}", adapter_ptr);
    let adapter: &mut Adapter = unsafe { &mut *(adapter_ptr as *mut Adapter) };

    let dev_ptr: *mut device = unsafe { iflib_get_dev(iflib_ptr) };
    e1000_println!("dev ptr: {:?}", dev_ptr);
    let dev = PciDevice {
        inner: unsafe { NonNull::new_unchecked(dev_ptr) },
    };

    e1000_println!("iflib ptr: {:?}", iflib_ptr);
    let iflib = IfLib {
        inner: unsafe { NonNull::new_unchecked(iflib_ptr) },
    };

    let iflib_shared_ptr: *mut if_softc_ctx = unsafe { iflib_get_softc_ctx(iflib_ptr) };
    e1000_println!("shared ctx ptr: {:?}", iflib_shared_ptr);
    let iflib_shared = IfLibShared {
        inner: unsafe { NonNull::new_unchecked(iflib_shared_ptr) },
    };

    let media_ptr: *mut ifmedia = unsafe { iflib_get_media(iflib_ptr) };
    e1000_println!("media ptr: {:?}", media_ptr);
    let media = IfMedia {
        inner: unsafe { NonNull::new_unchecked(media_ptr) },
    };

    let ifp_ptr: *mut ifnet = unsafe { iflib_get_ifp(iflib_ptr) };
    e1000_println!("ifnet ptr: {:?}", ifp_ptr);
    let ifp = IfNet {
        inner: unsafe { NonNull::new_unchecked(ifp_ptr) },
    };

    if let Some(e) = adapter.init_pre(dev, iflib, iflib_shared, media, ifp).err() {
        eprintln!(e);
        adapter.release();
        return 1;
    };

    if let Some(e) = adapter.attach_pre().err() {
        eprintln!(e);
        adapter.release();
        return 1;
    };
    0
}

#[no_mangle]
pub extern "C" fn rust_em_if_attach_post(iflib_ptr: *mut iflib_ctx) -> i32 {
    // printf("rust_em_if_attach_post\n\0");
    e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    match adapter.attach_post() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!(e);
            adapter.release();
            1
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_em_if_shutdown(iflib_ptr: *mut iflib_ctx) -> i32 {
    // printf("rust_em_if_shutdown\n\0");
    // log("INCOMPLETE: rust_em_if_shutdown\n\0");
    e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    incomplete!();
    1
}
#[no_mangle]
pub extern "C" fn rust_em_if_suspend(iflib_ptr: *mut iflib_ctx) -> i32 {
    // printf("rust_em_if_suspend\n\0");
    // log("INCOMPLETE: rust_em_if_suspend\n\0");
    e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    incomplete!();
    1
}
#[no_mangle]
pub extern "C" fn rust_em_if_resume(iflib_ptr: *mut iflib_ctx) -> i32 {
    // printf("rust_em_if_resume\n\0");
    // log("INCOMPLETE: rust_em_if_resume\n\0");
    e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    incomplete!();
    1
}
#[no_mangle]
pub extern "C" fn rust_em_if_init(iflib_ptr: *mut iflib_ctx) {
    // printf("rust_em_if_init\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    match adapter.init() {
        Ok(()) => {}
        Err(e) => {
            eprintln!(e);
            adapter.release();
        }
    }
}
#[no_mangle]
pub extern "C" fn rust_em_if_stop(iflib_ptr: *mut iflib_ctx) {
    // printf("rust_em_if_stop\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    match adapter.stop() {
        Ok(()) => {}
        Err(e) => {
            eprintln!(e);
            adapter.release();
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_em_if_tx_queues_alloc(
    iflib_ptr: *mut iflib_ctx,
    vaddrs: *mut caddr_t,
    paddrs: *mut u64,
    ntxqs: usize,
    ntxqsets: usize,
) -> i32 {
    // printf("rust_em_if_tx_queues_alloc\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    match adapter.tx_queues_alloc(vaddrs, paddrs, ntxqs, ntxqsets) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!(e);
            adapter.release();
            1
        }
    }
}
#[no_mangle]
pub extern "C" fn rust_em_if_rx_queues_alloc(
    iflib_ptr: *mut iflib_ctx,
    vaddrs: *mut caddr_t,
    paddrs: *mut u64,
    nrxqs: usize,
    nrxqsets: usize,
) -> i32 {
    // printf("rust_em_if_rx_queues_alloc\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    match adapter.rx_queues_alloc(vaddrs, paddrs, nrxqs, nrxqsets) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!(e);
            adapter.release();
            1
        }
    }
}
#[no_mangle]
pub extern "C" fn rust_em_if_queues_free(iflib_ptr: *mut iflib_ctx) {
    // printf("rust_em_if_queues_free\n\0");
    // log("INCOMPLETE: rust_em_if_queues_free\n\0");
    e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    incomplete!();
}
#[no_mangle]
pub extern "C" fn rust_em_if_get_counter(iflib_ptr: *mut iflib_ctx, ift: IftCounter) -> u64 {
    // printf("rust_em_if_get_counter\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    adapter.get_counter(ift)
}
#[no_mangle]
pub extern "C" fn rust_em_if_media_status(iflib_ptr: *mut iflib_ctx, mediareq: *mut ifmediareq) {
    // printf("rust_em_if_media_status\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    let mut ifmr = IfMediaReq {
        inner: unsafe { NonNull::new_unchecked(mediareq) },
    };
    adapter.media_status(&mut ifmr);
}
#[no_mangle]
pub extern "C" fn rust_em_if_media_change(iflib_ptr: *mut iflib_ctx) -> i32 {
    // printf("rust_em_if_media_change\n\0");
    // log("INCOMPLETE: rust_em_if_media_change\n\0");
    e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    incomplete!();
    1
}
#[no_mangle]
pub extern "C" fn rust_em_if_mtu_set(iflib_ptr: *mut iflib_ctx, mtu: u32) -> i32 {
    // printf("rust_em_if_mtu_set\n\0");
    // log("INCOMPLETE: rust_em_if_mtu_set\n\0");
    e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    incomplete!();
    // panic!();
    0
}
#[no_mangle]
pub extern "C" fn rust_em_if_timer(iflib_ptr: *mut iflib_ctx, qid: u16) {
    // printf("rust_em_if_timer\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    adapter.timer(qid);
}
#[no_mangle]
pub extern "C" fn rust_em_if_vlan_register(iflib_ptr: *mut iflib_ctx, vtag: u16) {
    // printf("rust_em_if_vlan_register\n\0");
    // log("INCOMPLETE: rust_em_if_vlan_register\n\0");
    e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    incomplete!();
    // panic!();
}
#[no_mangle]
pub extern "C" fn rust_em_if_vlan_unregister(iflib_ptr: *mut iflib_ctx, vtag: u16) {
    // printf("rust_em_if_vlan_unregister\n\0");
    // log("INCOMPLETE: rust_em_if_vlan_unregister\n\0");
    e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    incomplete!();
    // panic!();
}
#[no_mangle]
pub extern "C" fn rust_em_if_enable_intr(iflib_ptr: *mut iflib_ctx) {
    // printf("rust_em_if_enable_intr\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    adapter.enable_intr();
}
#[no_mangle]
pub extern "C" fn rust_em_if_disable_intr(iflib_ptr: *mut iflib_ctx) {
    // printf("rust_em_if_disable_intr\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    adapter.disable_intr();
}
#[no_mangle]
pub extern "C" fn rust_em_if_debug(iflib_ptr: *mut iflib_ctx) {
    // printf("rust_em_if_debug\n\0");
    // log("INCOMPLETE: rust_em_if_debug\n\0");
    e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    incomplete!();
    // panic!();
}
#[no_mangle]
pub extern "C" fn rust_em_if_rx_queue_intr_enable(iflib_ptr: *mut iflib_ctx, rxqid: u16) -> i32 {
    // printf("rust_em_if_rx_queue_intr_enable\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    adapter.rx_queue_intr_enable(rxqid);
    0
}
#[no_mangle]
pub extern "C" fn rust_em_if_tx_queue_intr_enable(iflib_ptr: *mut iflib_ctx, txqid: u16) -> i32 {
    // printf("rust_em_if_tx_queue_intr_enable\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    adapter.tx_queue_intr_enable(txqid);
    0
}
#[no_mangle]
pub extern "C" fn rust_em_if_multi_set(iflib_ptr: *mut iflib_ctx) {
    // printf("rust_em_if_multi_set\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    match adapter.multi_set() {
        Ok(_) => (),
        Err(e) => eprintln!(e),
    }
}
#[no_mangle]
pub extern "C" fn rust_em_if_update_admin_status(iflib_ptr: *mut iflib_ctx) {
    // printf("rust_em_if_update_admin_status\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    match adapter.update_admin_status() {
        Ok(_) => (),
        Err(e) => eprintln!(e),
    }
}
#[no_mangle]
pub extern "C" fn rust_em_if_set_promisc(iflib_ptr: *mut iflib_ctx, flags: u32) -> i32 {
    // printf("rust_em_if_set_promisc\n\0");
    e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    match adapter.set_promisc(flags) {
        Ok(_) => (),
        Err(e) => eprintln!(e),
    }
    0
}
#[no_mangle]
pub extern "C" fn rust_em_if_msix_intr_assign(iflib_ptr: *mut iflib_ctx, intr: isize) -> i32 {
    // printf("rust_em_if_msix_intr_assign\n\0");
    // log("INCOMPLETE: rust_em_if_msix_intr_assign\n\0");
    e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    incomplete!();
    // panic!();
    1
}
#[no_mangle]
pub extern "C" fn rust_em_if_led_func(iflib_ptr: *mut iflib_ctx, onoff: isize) {
    // printf("rust_em_if_led_func\n\0");
    // log("INCOMPLETE: rust_em_if_led_func\n\0");
    e1000_println!();
    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    incomplete!();
    // panic!();
}

#[no_mangle]
pub extern "C" fn rust_em_if_detach(iflib_ptr: *mut iflib_ctx) -> i32 {
    // printf("rust_em_if_detach\n\0");
    // log("INCOMPLETE: rust_em_if_detach\n\0");
    e1000_println!();
    incomplete!();

    let adapter: &mut Adapter = unsafe { &mut *(iflib_get_softc(iflib_ptr) as *mut Adapter) };
    let ret = match adapter.detach() {
        Ok(()) => {
            e1000_println!("Adapter detach done");
            adapter.release();
            0
        }
        Err(e) => {
            eprintln!(e);
            1
        }
    };

    // unsafe {
    // e1000_phy_hw_reset(&mut (*adapter).hw);
    // em_release_manageability(adapter);
    // em_release_hw_control(adapter);
    // em_free_pci_resources(ctx);
    // }

    return ret;
}

use kernel::sys::iflib_sys::qidx_t;
use kernel::sys::iflib_sys::if_rxd_info;
use kernel::sys::iflib_sys::if_rxd_update;
use sys::iflib::if_pkt_info;
use sys::iflib::if_txrx;

#[no_mangle]
pub static EM_TXRX: if_txrx = if_txrx {
    ift_txd_encap: Some(rust_em_isc_txd_encap),
    ift_txd_flush: Some(rust_em_isc_txd_flush),
    ift_txd_credits_update: Some(rust_em_isc_txd_credits_update),
    ift_rxd_available: Some(rust_em_isc_rxd_available),
    ift_rxd_pkt_get: Some(rust_em_isc_rxd_pkt_get),
    ift_rxd_refill: Some(rust_em_isc_rxd_refill),
    ift_rxd_flush: Some(rust_em_isc_rxd_flush),
    ift_legacy_intr: Some(rust_em_intr),
};

#[no_mangle]
pub static LEM_TXRX: if_txrx = if_txrx {
    ift_txd_encap: Some(rust_em_isc_txd_encap),
    ift_txd_flush: Some(rust_em_isc_txd_flush),
    ift_txd_credits_update: Some(rust_em_isc_txd_credits_update),
    ift_rxd_available: Some(rust_lem_isc_rxd_available),
    ift_rxd_pkt_get: Some(rust_lem_isc_rxd_pkt_get),
    ift_rxd_refill: Some(rust_lem_isc_rxd_refill),
    ift_rxd_flush: Some(rust_em_isc_rxd_flush),
    ift_legacy_intr: Some(rust_em_intr),
};

#[no_mangle]
pub unsafe extern "C" fn rust_em_isc_txd_encap(
    adapter_ptr: *mut ::kernel::sys::raw::c_void,
    pi_ptr: *mut if_pkt_info,
) -> ::kernel::sys::raw::c_int {
    // printf("rust_em_isc_txd_encap\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = &mut *(adapter_ptr as *mut Adapter);
    let mut pi = IfPacketInfo {
        inner: NonNull::new_unchecked(pi_ptr),
    };
    adapter.em_txd_encap(&mut pi)
}

#[no_mangle]
pub unsafe extern "C" fn rust_em_isc_txd_flush(
    adapter_ptr: *mut ::kernel::sys::raw::c_void,
    txqid: u16,
    pidx: u16,
) {
    // printf("rust_em_isc_txd_flush\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = &mut *(adapter_ptr as *mut Adapter);
    adapter.em_txd_flush(txqid, pidx);
}

#[no_mangle]
pub unsafe extern "C" fn rust_em_isc_txd_credits_update(
    adapter_ptr: *mut ::kernel::sys::raw::c_void,
    txqid: u16,
    clear: bool,
) -> ::kernel::sys::raw::c_int {
    // printf("rust_em_isc_txd_credits_update\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = &mut *(adapter_ptr as *mut Adapter);
    adapter.em_txd_credits_update(txqid, clear)
}

#[no_mangle]
pub unsafe extern "C" fn rust_em_isc_rxd_available(
    adapter_ptr: *mut ::kernel::sys::raw::c_void,
    rxqid: u16,
    idx: u16,
    budget: u16,
) -> ::kernel::sys::raw::c_int {
    // printf("rust_lem_isc_rxd_available\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = &mut *(adapter_ptr as *mut Adapter);
    adapter.em_rxd_available(rxqid, idx, budget)
}

#[no_mangle]
pub unsafe extern "C" fn rust_lem_isc_rxd_available(
    adapter_ptr: *mut ::kernel::sys::raw::c_void,
    rxqid: u16,
    idx: u16,
    budget: u16,
) -> ::kernel::sys::raw::c_int {
    // printf("rust_lem_isc_rxd_available\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = &mut *(adapter_ptr as *mut Adapter);
    adapter.lem_rxd_available(rxqid, idx, budget)
}

#[no_mangle]
pub unsafe extern "C" fn rust_em_isc_rxd_pkt_get(
    adapter_ptr: *mut ::kernel::sys::raw::c_void,
    ri_ptr: *mut if_rxd_info,
) -> ::kernel::sys::raw::c_int {
    // printf("rust_lem_isc_rxd_pkt_get\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = &mut *(adapter_ptr as *mut Adapter);
    let mut ri = IfRxdInfo {
        inner: NonNull::new_unchecked(ri_ptr),
    };
    adapter.em_rxd_pkt_get(&mut ri)
}

#[no_mangle]
pub unsafe extern "C" fn rust_lem_isc_rxd_pkt_get(
    adapter_ptr: *mut ::kernel::sys::raw::c_void,
    ri_ptr: *mut if_rxd_info,
) -> ::kernel::sys::raw::c_int {
    // printf("rust_lem_isc_rxd_pkt_get\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = &mut *(adapter_ptr as *mut Adapter);
    let mut ri = IfRxdInfo {
        inner: NonNull::new_unchecked(ri_ptr),
    };
    adapter.lem_rxd_pkt_get(&mut ri)
}

#[no_mangle]
pub unsafe extern "C" fn rust_em_isc_rxd_refill(
    adapter_ptr: *mut ::kernel::sys::raw::c_void,
    iru_ptr: *mut if_rxd_update,
) {
    // printf("rust_lem_isc_rxd_refill\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = &mut *(adapter_ptr as *mut Adapter);
    let mut iru = IfRxdUpdate {
        inner: NonNull::new_unchecked(iru_ptr),
    };
    adapter.em_rxd_refill(&mut iru);
}

#[no_mangle]
pub unsafe extern "C" fn rust_lem_isc_rxd_refill(
    adapter_ptr: *mut ::kernel::sys::raw::c_void,
    iru_ptr: *mut if_rxd_update,
) {
    // printf("rust_lem_isc_rxd_refill\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = &mut *(adapter_ptr as *mut Adapter);
    let mut iru = IfRxdUpdate {
        inner: NonNull::new_unchecked(iru_ptr),
    };
    adapter.lem_rxd_refill(&mut iru);
}

#[no_mangle]
pub unsafe extern "C" fn rust_em_isc_rxd_flush(
    adapter_ptr: *mut ::kernel::sys::raw::c_void,
    rxqid: u16,
    flid: u8,
    pidx: u16,
) {
    // printf("rust_em_isc_rxd_flush\n\0");
    // e1000_println!();
    let adapter: &mut Adapter = &mut *(adapter_ptr as *mut Adapter);
    adapter.em_rxd_flush(rxqid, flid, pidx);
}

#[no_mangle]
pub unsafe extern "C" fn rust_em_intr(
    adapter_ptr: *mut ::kernel::sys::raw::c_void,
) -> ::kernel::sys::raw::c_int {
    // printf("rust_em_intr\n\0");
    // kernel::sys::systm_sys::log(0, "rust_em_intr".as_ptr() as *const i8);
    // e1000_println!();
    let adapter: &mut Adapter = &mut *(adapter_ptr as *mut Adapter);
    adapter.em_intr()
}
