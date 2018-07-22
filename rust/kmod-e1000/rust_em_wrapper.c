/*-
 * SPDX-License-Identifier: BSD-2-Clause
 *
 * Copyright (c) 2016 Matthew Macy <mmacy@mattmacy.io>
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
 * OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 */

/* $FreeBSD$ */
#include "c-src/if_em.h"
#include <sys/sbuf.h>
#include <machine/_inttypes.h>

/*********************************************************************
 *  For Rust interface
 *********************************************************************/
extern int	rust_em_adapter_size(void);

extern int	rust_em_if_attach_pre(if_ctx_t ctx);
extern int	rust_em_if_attach_post(if_ctx_t ctx);
extern int	rust_em_if_detach(if_ctx_t ctx);
extern int	rust_em_if_shutdown(if_ctx_t ctx);
extern int	rust_em_if_suspend(if_ctx_t ctx);
extern int	rust_em_if_resume(if_ctx_t ctx);
extern int	rust_em_if_tx_queues_alloc(if_ctx_t ctx, caddr_t *vaddrs, uint64_t *paddrs, int ntxqs, int ntxqsets);
extern int	rust_em_if_rx_queues_alloc(if_ctx_t ctx, caddr_t *vaddrs, uint64_t *paddrs, int nrxqs, int nrxqsets);
extern void	rust_em_if_queues_free(if_ctx_t ctx);
extern uint64_t	rust_em_if_get_counter(if_ctx_t, ift_counter);
extern void	rust_em_if_init(if_ctx_t ctx);
extern void	rust_em_if_stop(if_ctx_t ctx);
extern void	rust_em_if_media_status(if_ctx_t, struct ifmediareq *);
extern int	rust_em_if_media_change(if_ctx_t ctx);
extern int	rust_em_if_mtu_set(if_ctx_t ctx, uint32_t mtu);
extern void	rust_em_if_timer(if_ctx_t ctx, uint16_t qid);
extern void	rust_em_if_vlan_register(if_ctx_t ctx, u16 vtag);
extern void	rust_em_if_vlan_unregister(if_ctx_t ctx, u16 vtag);
extern void	rust_em_if_enable_intr(if_ctx_t ctx);
extern void	rust_em_if_disable_intr(if_ctx_t ctx);
extern int	rust_em_if_rx_queue_intr_enable(if_ctx_t ctx, uint16_t rxqid);
extern int	rust_em_if_tx_queue_intr_enable(if_ctx_t ctx, uint16_t txqid);
extern void	rust_em_if_multi_set(if_ctx_t ctx);
extern void	rust_em_if_update_admin_status(if_ctx_t ctx);
extern void	rust_em_if_debug(if_ctx_t ctx);
extern int	rust_em_if_set_promisc(if_ctx_t ctx, int flags);
extern int	rust_em_if_msix_intr_assign(if_ctx_t, int);
extern void	rust_em_if_led_func(if_ctx_t ctx, int onoff);


/*********************************************************************
 *  Function prototypes:
 *********************************************************************/

static void *rem_register(device_t dev);


/*********************************************************************
 *  Driver version:
 *********************************************************************/
char rem_driver_version[] = "7.6.1-k";

/*********************************************************************
 *  PCI Device ID Table
 *
 *  Used by probe to select devices to load on
 *  Last field stores an index into e1000_strings
 *  Last entry must be all 0s
 *
 *  { Vendor ID, Device ID, SubVendor ID, SubDevice ID, String Index }
 *********************************************************************/

static pci_vendor_info_t rem_vendor_info_array[] =
{
	/* bhyve and virtualbox */
	PVID(0x8086, E1000_DEV_ID_82545EM_COPPER, "Intel(R) PRO/1000 Network Connection"),
	
	/* Dell Latitude E7450 */
	PVID(0x8086, E1000_DEV_ID_PCH_I218_LM3, "Intel(R) PRO/1000 Network Connection"),

	/* Dell Latitude E7270 (Porting to Rust 80% complete) */
	PVID(0x8086, E1000_DEV_ID_PCH_SPT_I219_LM, "Intel(R) PRO/1000 Network Connection"),

	/* required last entry */
	PVID_END
};


/*********************************************************************
 *  FreeBSD Device Interface Entry Points
 *********************************************************************/
static device_method_t rem_methods[] = {
	/* Device interface */
	DEVMETHOD(device_register, rem_register),
	DEVMETHOD(device_probe, iflib_device_probe),
	DEVMETHOD(device_attach, iflib_device_attach),
	DEVMETHOD(device_detach, iflib_device_detach),
	DEVMETHOD(device_shutdown, iflib_device_shutdown),
	DEVMETHOD(device_suspend, iflib_device_suspend),
	DEVMETHOD(device_resume, iflib_device_resume),
	DEVMETHOD_END
};

static driver_t rem_driver = {
	"rem", rem_methods, sizeof(struct adapter),
};

static devclass_t rem_devclass;
DRIVER_MODULE(rem, pci, rem_driver, rem_devclass, 0, 0);

MODULE_DEPEND(rem, pci, 1, 1, 1);
MODULE_DEPEND(rem, ether, 1, 1, 1);
MODULE_DEPEND(rem, iflib, 1, 1, 1);
MODULE_DEPEND(rem, rustkpi, 1, 1, 1);

IFLIB_PNP_INFO(pci, rem, rem_vendor_info_array);


static device_method_t rem_if_methods[] = {
	DEVMETHOD(ifdi_attach_pre, rust_em_if_attach_pre),
	DEVMETHOD(ifdi_attach_post, rust_em_if_attach_post),
	DEVMETHOD(ifdi_detach, rust_em_if_detach),
	DEVMETHOD(ifdi_shutdown, rust_em_if_shutdown),
	DEVMETHOD(ifdi_suspend, rust_em_if_suspend),
	DEVMETHOD(ifdi_resume, rust_em_if_resume),
	DEVMETHOD(ifdi_init, rust_em_if_init),
	DEVMETHOD(ifdi_stop, rust_em_if_stop),
	DEVMETHOD(ifdi_msix_intr_assign, rust_em_if_msix_intr_assign),
	DEVMETHOD(ifdi_intr_enable, rust_em_if_enable_intr),
	DEVMETHOD(ifdi_intr_disable, rust_em_if_disable_intr),
	DEVMETHOD(ifdi_tx_queues_alloc, rust_em_if_tx_queues_alloc),
	DEVMETHOD(ifdi_rx_queues_alloc, rust_em_if_rx_queues_alloc),
	DEVMETHOD(ifdi_queues_free, rust_em_if_queues_free),
	DEVMETHOD(ifdi_update_admin_status, rust_em_if_update_admin_status),
	DEVMETHOD(ifdi_multi_set, rust_em_if_multi_set),
	DEVMETHOD(ifdi_media_status, rust_em_if_media_status),
	DEVMETHOD(ifdi_media_change, rust_em_if_media_change),
	DEVMETHOD(ifdi_mtu_set, rust_em_if_mtu_set),
	DEVMETHOD(ifdi_promisc_set, rust_em_if_set_promisc),
	DEVMETHOD(ifdi_timer, rust_em_if_timer),
	DEVMETHOD(ifdi_vlan_register, rust_em_if_vlan_register),
	DEVMETHOD(ifdi_vlan_unregister, rust_em_if_vlan_unregister),
	DEVMETHOD(ifdi_get_counter, rust_em_if_get_counter),
	DEVMETHOD(ifdi_led_func, rust_em_if_led_func),
	DEVMETHOD(ifdi_rx_queue_intr_enable, rust_em_if_rx_queue_intr_enable),
	DEVMETHOD(ifdi_tx_queue_intr_enable, rust_em_if_tx_queue_intr_enable),
	DEVMETHOD(ifdi_debug, rust_em_if_debug),
	DEVMETHOD_END
};

/*
 * note that if (adapter->msix_mem) is replaced by:
 * if (adapter->intr_type == IFLIB_INTR_MSIX)
 */
static driver_t rem_if_driver = {
	"rem_if", rem_if_methods, sizeof(struct adapter)
};

static struct if_shared_ctx rem_sctx_init = {
	.isc_magic = IFLIB_MAGIC,
	.isc_q_align = PAGE_SIZE,
	.isc_tx_maxsize = EM_TSO_SIZE,
	.isc_tx_maxsegsize = PAGE_SIZE,
	.isc_rx_maxsize = MJUM9BYTES,
	.isc_rx_nsegments = 1,
	.isc_rx_maxsegsize = MJUM9BYTES,
	.isc_nfl = 1,
	.isc_nrxqs = 1,
	.isc_ntxqs = 1,
	.isc_admin_intrcnt = 1,
	.isc_vendor_info = rem_vendor_info_array,
	.isc_driver_version = rem_driver_version,
	.isc_driver = &rem_if_driver,
	.isc_flags = IFLIB_NEED_SCRATCH | IFLIB_TSO_INIT_IP | IFLIB_NEED_ZERO_CSUM,

	.isc_nrxd_min = {EM_MIN_RXD},
	.isc_ntxd_min = {EM_MIN_TXD},
	.isc_nrxd_max = {EM_MAX_RXD},
	.isc_ntxd_max = {EM_MAX_TXD},
	.isc_nrxd_default = {EM_DEFAULT_RXD},
	.isc_ntxd_default = {EM_DEFAULT_TXD},
};

if_shared_ctx_t rem_sctx = &rem_sctx_init;

static void *
rem_register(device_t dev)
{
	/*
	 * Memory for our adapter object is malloc'd in iflib, 
	 * get the size from Rust code here.
	 */
	int size = rust_em_adapter_size();
	rem_sctx_init.isc_driver->size = size;

	return (rem_sctx);
}
