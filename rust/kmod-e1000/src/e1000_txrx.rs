use kernel;
use kernel::ptr::Unique;

use kernel::sys::raw::*;
use kernel::prelude::v1::*;

use kernel::sys::kernel_sys::caddr_t;
use kernel::sys::iflib_sys::bus_dma_segment;
use kernel::sys::iflib_sys::bus_dma_segment_t;
use kernel::sys::iflib_sys::device_t;
use kernel::sys::iflib_sys::if_rxd_frag;
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

use adapter::*;
use e1000_regs::*;
use e1000_osdep::*;
use e1000_manage;
use e1000_82540;
use e1000_82541;
use e1000_82542;
use e1000_82543;
use e1000_nvm;
use e1000_phy;
use e1000_mac;

use log;
use printf;

// #define DEFAULT_ITR		(1000000000/(MAX_INTS_PER_SEC * 256))

const MAX_INTS_PER_SEC: u32 = 8000;
const DEFAULT_ITR: u32 = 1000000000 / (MAX_INTS_PER_SEC * 256);

const TSO_WORKAROUND: u64 = 4;

impl Adapter {
    // em_tso_setup(struct adapter *adapter, if_pkt_info_t pi, u32 *txd_upper, u32 *txd_lower)
    pub fn tso_setup(
        &mut self,
        pi: &IfPacketInfo,
        txd_upper: &mut u32,
        txd_lower: &mut u32,
    ) -> u16 {
        e1000_println!();

        // if_softc_ctx_t scctx = adapter->shared;
        // struct em_tx_queue *que = &adapter->tx_queues[pi->ipi_qsidx];
        // struct tx_ring *txr = &que->txr;
        // struct e1000_context_desc *TXD;
        // int cur, hdr_len;

        let txq: &mut TxQueue = &mut self.tx_queues[pi.ipi_qsidx as usize];
        let txr: &mut TxRing = &mut txq.txr;

        // hdr_len = pi->ipi_ehdrlen + pi->ipi_ip_hlen + pi->ipi_tcp_hlen;
        // *txd_lower = (E1000_TXD_CMD_DEXT |	/* Extended descr type */
        // 	      E1000_TXD_DTYP_D |	/* Data descr type */
        // 	      E1000_TXD_CMD_TSE);	/* Do TSE on this packet */

        let hdr_len = pi.ipi_ehdrlen + pi.ipi_ip_hlen + pi.ipi_tcp_hlen;
        *txd_lower = E1000_TXD_CMD_DEXT | E1000_TXD_DTYP_D | E1000_TXD_CMD_TSE;

        /* IP and/or TCP header checksum calculation and insertion. */
        // *txd_upper = (E1000_TXD_POPTS_IXSM | E1000_TXD_POPTS_TXSM) << 8;
        *txd_upper = (E1000_TXD_POPTS_IXSM | E1000_TXD_POPTS_TXSM) << 8;

        // cur = pi->ipi_pidx;
        // TXD = (struct e1000_context_desc *)&txr->tx_base[cur];
        let mut cur: u16 = pi.ipi_pidx;
        {
            // limit scope where we mut borrow txr
            let len = self.iflib_shared.isc_ntxd[0] as usize;
            let txd_slice = txr.txd_context_desc_slice(len);

            let txd: &mut e1000_context_desc = &mut txd_slice[cur as usize];
            /*
             * Start offset for header checksum calculation.
             * End offset for header checksum calculation.
             * Offset of place put the checksum.
             */
            // TXD->lower_setup.ip_fields.ipcss = pi->ipi_ehdrlen;
            // TXD->lower_setup.ip_fields.ipcse =
            //     htole16(pi->ipi_ehdrlen + pi->ipi_ip_hlen - 1);
            // TXD->lower_setup.ip_fields.ipcso = pi->ipi_ehdrlen + offsetof(struct ip, ip_sum);
            unsafe {
                txd.lower_setup.ip_fields.ipcss = pi.ipi_ehdrlen;
                txd.lower_setup.ip_fields.ipcse = (pi.ipi_ehdrlen + pi.ipi_ip_hlen - 1) as u16;
                txd.lower_setup.ip_fields.ipcso = pi.ipi_ehdrlen + offset_of!(ip, ip_sum) as u8;
                assert!(offset_of!(ip, ip_sum) == 10);
            }

            /*
             * Start offset for payload checksum calculation.
             * End offset for payload checksum calculation.
             * Offset of place to put the checksum.
             */
            // TXD->upper_setup.tcp_fields.tucss = pi->ipi_ehdrlen + pi->ipi_ip_hlen;
            // TXD->upper_setup.tcp_fields.tucse = 0;
            // TXD->upper_setup.tcp_fields.tucso =
            //     pi->ipi_ehdrlen + pi->ipi_ip_hlen + offsetof(struct tcphdr, th_sum);
            unsafe {
                txd.upper_setup.tcp_fields.tucss = pi.ipi_ehdrlen + pi.ipi_ip_hlen;
                txd.upper_setup.tcp_fields.tucse = 0;
                txd.upper_setup.tcp_fields.tucso =
                    pi.ipi_ehdrlen + pi.ipi_ip_hlen + offset_of!(tcphdr, th_sum) as u8;
                assert!(offset_of!(tcphdr, th_sum) == 16);
            }
            /*
             * Payload size per packet w/o any headers.
             * Length of all headers up to payload.
             */
            // TXD->tcp_seg_setup.fields.mss = htole16(pi->ipi_tso_segsz);
            // TXD->tcp_seg_setup.fields.hdr_len = hdr_len;
            unsafe {
                txd.tcp_seg_setup.fields.mss = pi.ipi_tso_segsz;
                txd.tcp_seg_setup.fields.hdr_len = hdr_len;
            }

            // TXD->cmd_and_length = htole32(adapter->txd_cmd |
            // 			E1000_TXD_CMD_DEXT |	/* Extended descr */
            // 			E1000_TXD_CMD_TSE |	/* TSE context */
            // 			E1000_TXD_CMD_IP |	/* Do IP csum */
            // 			E1000_TXD_CMD_TCP |	/* Do TCP checksum */
            // 			      (pi->ipi_len - hdr_len)); /* Total len */
            // txr->tx_tso = TRUE;
            txd.cmd_and_length = self.txd_cmd |
            E1000_TXD_CMD_DEXT |	/* Extended descr */
            E1000_TXD_CMD_TSE |	/* TSE context */
            E1000_TXD_CMD_IP |	/* Do IP csum */
            E1000_TXD_CMD_TCP |	/* Do TCP checksum */
            (pi.ipi_len - hdr_len as u32); /* Total len */

            // if (++cur == scctx->isc_ntxd[0]) {
            // 	cur = 0;
            // }
            cur += 1;
            if cur == self.iflib_shared.isc_ntxd[0] as u16 {
                cur = 0;
            }
        }
        txr.tx_tso = true;

        // DPRINTF(iflib_get_dev(adapter->ctx), "%s: pidx: %d cur: %d\n", __FUNCTION__, pi->ipi_pidx, cur);
        e1000_println!("pidx = {}, cur = {}", pi.ipi_pidx, cur);
        // return (cur);
        // incomplete!();
        cur
    }
    pub fn transmit_checksum_setup(
        &mut self,
        pi: &IfPacketInfo,
        txd_upper: &mut u32,
        txd_lower: &mut u32,
    ) -> u16 {
        e1000_println!();
        //  struct e1000_context_desc *TXD = NULL;
        // if_softc_ctx_t scctx = adapter->shared;
        // struct em_tx_queue *que = &adapter->tx_queues[pi->ipi_qsidx];
        // struct tx_ring *txr = &que->txr;
        // int csum_flags = pi->ipi_csum_flags;
        // int cur, hdr_len;
        // u32 cmd;
        let mut cur;
        let hdr_len;
        let mut cmd;
        let csum_flags = pi.ipi_csum_flags;

        let txq: &mut TxQueue = &mut self.tx_queues[pi.ipi_qsidx as usize];
        let txr: &mut TxRing = &mut txq.txr;

        // cur = pi->ipi_pidx;
        // hdr_len = pi->ipi_ehdrlen + pi->ipi_ip_hlen;
        // cmd = adapter->txd_cmd;
        cur = pi.ipi_pidx;
        hdr_len = pi.ipi_ehdrlen + pi.ipi_ip_hlen;
        cmd = self.txd_cmd;

        /*
         * The 82574L can only remember the *last* context used
         * regardless of queue that it was use for.  We cannot reuse
         * contexts on this hardware platform and must generate a new
         * context every time.  82574L hardware spec, section 7.2.6,
         * second note.
         */
        // if (DONT_FORCE_CTX &&
        //     adapter->tx_num_queues == 1 &&
        //     txr->csum_lhlen == pi->ipi_ehdrlen &&
        //     txr->csum_iphlen == pi->ipi_ip_hlen &&
        //     txr->csum_flags == csum_flags) {
        // 	/*
        // 	 * Same csum offload context as the previous packets;
        // 	 * just return.
        // 	 */
        // 	*txd_upper = txr->csum_txd_upper;
        // 	*txd_lower = txr->csum_txd_lower;
        // 	return (cur);
        // }
        const DONT_FORCE_CTX: bool = true;

        if DONT_FORCE_CTX && self.iflib_shared.isc_ntxqsets == 1
            && txr.csum_lhlen == pi.ipi_ehdrlen as i32
            && txr.csum_iphlen == pi.ipi_ip_hlen as i32
            && txr.csum_flags == csum_flags as i32
        {
            // println!("transmit_checksum_setup: return early");
            *txd_upper = txr.csum_txd_upper;
            *txd_lower = txr.csum_txd_lower;
            return cur;
        }
        // TXD = (struct e1000_context_desc *)&txr->tx_base[cur];
        // if (csum_flags & CSUM_IP) {
        // 	*txd_upper |= E1000_TXD_POPTS_IXSM << 8;
        // 	/*
        // 	 * Start offset for header checksum calculation.
        // 	 * End offset for header checksum calculation.
        // 	 * Offset of place to put the checksum.
        // 	 */
        // 	TXD->lower_setup.ip_fields.ipcss = pi->ipi_ehdrlen;
        // 	TXD->lower_setup.ip_fields.ipcse = htole16(hdr_len);
        // 	TXD->lower_setup.ip_fields.ipcso = pi->ipi_ehdrlen + offsetof(struct ip, ip_sum);
        // 	cmd |= E1000_TXD_CMD_IP;
        // }
        {
            // limit borrow scope
            let txd_slice: &mut [e1000_context_desc] =
                txr.txd_context_desc_slice(self.iflib_shared.isc_ntxd[0] as usize);
            let txd: &mut e1000_context_desc = &mut txd_slice[cur as usize];
            if csum_flags & CSUM_IP != 0 {
                *txd_upper |= E1000_TXD_POPTS_IXSM << 8;
                unsafe {
                    txd.lower_setup.ip_fields.ipcss = pi.ipi_ehdrlen;
                    txd.lower_setup.ip_fields.ipcse = hdr_len as u16;
                    txd.lower_setup.ip_fields.ipcso = pi.ipi_ehdrlen + offset_of!(ip, ip_sum) as u8;
                }
                cmd |= E1000_TXD_CMD_IP;
            }
            // if (csum_flags & (CSUM_TCP|CSUM_UDP)) {
            // 	uint8_t tucso;

            // 	*txd_upper |= E1000_TXD_POPTS_TXSM << 8;
            // 	*txd_lower = E1000_TXD_CMD_DEXT | E1000_TXD_DTYP_D;

            // 	if (csum_flags & CSUM_TCP) {
            // 		tucso = hdr_len + offsetof(struct tcphdr, th_sum);
            // 		cmd |= E1000_TXD_CMD_TCP;
            // 	} else
            // 		tucso = hdr_len + offsetof(struct udphdr, uh_sum);
            // 	TXD->upper_setup.tcp_fields.tucss = hdr_len;
            // 	TXD->upper_setup.tcp_fields.tucse = htole16(0);
            // 	TXD->upper_setup.tcp_fields.tucso = tucso;
            // }
            if csum_flags & (CSUM_TCP | CSUM_UDP) != 0 {
                let tucso: u8;
                *txd_upper |= E1000_TXD_POPTS_TXSM << 8;
                *txd_lower = E1000_TXD_CMD_DEXT | E1000_TXD_DTYP_D;

                if csum_flags & CSUM_TCP != 0 {
                    tucso = hdr_len + unsafe { offset_of!(tcphdr, th_sum) } as u8;
                    cmd |= E1000_TXD_CMD_TCP;
                } else {
                    tucso = hdr_len + unsafe { offset_of!(udphdr, uh_sum) } as u8;
                }
                unsafe {
                    txd.upper_setup.tcp_fields.tucss = hdr_len;
                    txd.upper_setup.tcp_fields.tucse = 0;
                    txd.upper_setup.tcp_fields.tucso = tucso;
                }
            }
            // TXD->tcp_seg_setup.data = htole32(0);
            // TXD->cmd_and_length =
            // 	htole32(E1000_TXD_CMD_IFCS | E1000_TXD_CMD_DEXT | cmd);
            txd.tcp_seg_setup.data = 0;
            txd.cmd_and_length = E1000_TXD_CMD_IFCS | E1000_TXD_CMD_DEXT | cmd;
        }
        // txr->csum_lhlen = pi->ipi_ehdrlen;
        // txr->csum_iphlen = pi->ipi_ip_hlen;
        // txr->csum_flags = csum_flags;
        // txr->csum_txd_upper = *txd_upper;
        // txr->csum_txd_lower = *txd_lower;
        txr.csum_lhlen = pi.ipi_ehdrlen as i32;
        txr.csum_iphlen = pi.ipi_ip_hlen as i32;
        txr.csum_flags = csum_flags as i32;
        txr.csum_txd_upper = *txd_upper;
        txr.csum_txd_lower = *txd_lower;

        // if (++cur == scctx->isc_ntxd[0]) {
        // 	cur = 0;
        // }
        cur += 1;
        if cur == self.iflib_shared.isc_ntxd[0] as u16 {
            cur = 0;
        }

        // DPRINTF(iflib_get_dev(adapter->ctx), "checksum_setup csum_flags=%x txd_upper=%x txd_lower=%x hdr_len=%d cmd=%x\n",
        // 	      csum_flags, *txd_upper, *txd_lower, hdr_len, cmd);
        e1000_println!(
            "checksum_setup csum_flags = 0x{:x}, txd_upper = 0x{:x}, txd_lower = 0x{:x}, hdr_len = 0x{:x}, cmd = 0x{:x}",
            csum_flags,
            txd_upper,
            txd_lower,
            hdr_len,
            cmd
        );
        // return (cur);
        cur
    }

    pub fn rx_queue_intr_enable(&mut self, rxqid: u16) {
        e1000_println!();

        // E1000_WRITE_REG(&adapter->hw, E1000_IMS, rxq->eims);
        let eims = self.rx_queues[rxqid as usize].eims;
        do_write_register(self, E1000_IMS, eims);
    }

    pub fn tx_queue_intr_enable(&mut self, txqid: u16) {
        e1000_println!();

        // E1000_WRITE_REG(&adapter->hw, E1000_IMS, txq->eims);
        let eims = self.tx_queues[txqid as usize].eims;
        do_write_register(self, E1000_IMS, eims);
    }
}

impl IfTxRx for Adapter {
    #[no_mangle]
    fn em_txd_encap(&mut self, pi: &mut IfPacketInfo) -> i32 {
        // log("txrx txd_encap\n\0");
        e1000_println!();
        // struct adapter *sc = arg;
        // if_softc_ctx_t scctx = sc->shared;
        // struct em_tx_queue *que = &sc->tx_queues[pi->ipi_qsidx];
        // struct tx_ring *txr = &que->txr;
        // bus_dma_segment_t *segs = pi->ipi_segs;
        // int nsegs = pi->ipi_nsegs;
        // int csum_flags = pi->ipi_csum_flags;
        // int i, j, first, pidx_last;
        // u32 txd_flags, txd_upper = 0, txd_lower = 0;

        let nsegs = pi.ipi_nsegs as usize;
        let segs: &mut [bus_dma_segment] =
            unsafe { kernel::slice::from_raw_parts_mut(pi.ipi_segs, nsegs as usize) };
        let csum_flags: u32 = pi.ipi_csum_flags;

        // struct e1000_tx_desc *ctxd = NULL;
        // bool do_tso, tso_desc;
        // qidx_t ntxd;

        // txd_flags = pi->ipi_flags & IPI_TX_INTR ? E1000_TXD_CMD_RS : 0;
        // i = first = pi->ipi_pidx;
        // do_tso = (csum_flags & CSUM_TSO);
        // tso_desc = FALSE;
        // ntxd = scctx->isc_ntxd[0];

        let txd_flags = match pi.ipi_flags & IPI_TX_INTR as u16 != 0 {
            true => E1000_TXD_CMD_RS,
            false => 0,
        };
        let first = pi.ipi_pidx;
        let mut i = first;
        let do_tso = (csum_flags & CSUM_TSO) != 0;
        let mut tso_desc = false;
        let ntxd = self.iflib_shared.isc_ntxd[0];

        let mut txd_upper: u32 = 0;
        let mut txd_lower: u32 = 0;

        {
            let txq: &mut TxQueue = &mut self.tx_queues[pi.ipi_qsidx as usize];
            let txr: &mut TxRing = &mut txq.txr;

            /*
             * TSO Hardware workaround, if this packet is not
             * TSO, and is only a single descriptor long, and
             * it follows a TSO burst, then we need to add a
             * sentinel descriptor to prevent premature writeback.
             */
            // if ((!do_tso) && (txr->tx_tso == TRUE)) {
            // 	if (nsegs == 1)
            // 		tso_desc = TRUE;
            // 	txr->tx_tso = FALSE;
            // }
            if !do_tso && txr.tx_tso {
                if nsegs == 1 {
                    tso_desc = true;
                }
                txr.tx_tso = false;
            }
        }

        /* Do hardware assists */
        // if (do_tso) {
        // 	i = em_tso_setup(sc, pi, &txd_upper, &txd_lower);
        // 	tso_desc = TRUE;
        // } else if (csum_flags & EM_CSUM_OFFLOAD) {
        // 	i = em_transmit_checksum_setup(sc, pi, &txd_upper, &txd_lower);
        // }
        if do_tso {
            i = self.tso_setup(pi, &mut txd_upper, &mut txd_lower);
            tso_desc = true;
        } else if csum_flags & EM_CSUM_OFFLOAD != 0 {
            i = self.transmit_checksum_setup(pi, &mut txd_upper, &mut txd_lower);
        }

        // if (pi->ipi_mflags & M_VLANTAG) {
        // 	/* Set the vlan id. */
        // 	txd_upper |= htole16(pi->ipi_vtag) << 16;
        // 	/* Tell hardware to add tag */
        // 	txd_lower |= htole32(E1000_TXD_CMD_VLE);
        // }
        if pi.ipi_mflags & M_VLANTAG as u8 != 0 {
            txd_upper |= (pi.ipi_vtag as u32) << 16;
            txd_lower |= E1000_TXD_CMD_VLE;
        }

        let txq: &mut TxQueue = &mut self.tx_queues[pi.ipi_qsidx as usize];
        let txr: &mut TxRing = &mut txq.txr;

        // DPRINTF(iflib_get_dev(sc->ctx), "encap: set up tx: nsegs=%d first=%d i=%d\n", nsegs, first, i);
        // /* XXX adapter->pcix_82544 -- lem_fill_descriptors */
        e1000_println!("set up tx: nsegs={}, first={}, i={}", nsegs, first, i);

        // /* Set up our transmit descriptors */
        // for (j = 0; j < nsegs; j++) {
        // 	bus_size_t seg_len;
        // 	bus_addr_t seg_addr;
        // 	uint32_t cmd;

        let mut pidx_last = first;
        {
            // limit borrow of txr to this scope
            let txd_slice: &mut [e1000_tx_desc] = txr.txd_tx_desc_slice(ntxd as usize);

            for j in 0..nsegs {
                // 	ctxd = &txr->tx_base[i];
                // 	seg_addr = segs[j].ds_addr;
                // 	seg_len = segs[j].ds_len;
                // 	cmd = E1000_TXD_CMD_IFCS | sc->txd_cmd;
                let seg_addr = segs[j].ds_addr;
                let mut seg_len = segs[j].ds_len;
                let cmd = E1000_TXD_CMD_IFCS | self.txd_cmd;

                /*
                 * TSO Workaround:
                 * If this is the last descriptor, we want to
                 * split it so we have a small final sentinel
                 */
                // 	if (tso_desc && (j == (nsegs - 1)) && (seg_len > 8)) {
                if tso_desc && (j == (nsegs - 1)) && (seg_len > 8) {
                    // seg_len -= TSO_WORKAROUND;
                    // ctxd->buffer_addr = htole64(seg_addr);
                    // ctxd->lower.data = htole32(cmd | txd_lower | seg_len);
                    // ctxd->upper.data = htole32(txd_upper);
                    {
                        // limit borrow to this scope
                        let ctxd: &mut e1000_tx_desc = &mut txd_slice[i as usize];
                        // e1000_println!("1: did make ctxd i = {}", i);
                        seg_len -= TSO_WORKAROUND;
                        ctxd.buffer_addr = seg_addr;
                        ctxd.lower.data = cmd | txd_lower | seg_len as u32;
                        ctxd.upper.data = txd_upper;
                    }
                    // if (++i == scctx->isc_ntxd[0])
                    //     i = 0;
                    i += 1;
                    if i == ntxd as u16 {
                        i = 0;
                    }

                    /* Now make the sentinel */
                    // ctxd = &txr->tx_base[i];
                    // ctxd->buffer_addr = htole64(seg_addr + seg_len);
                    // ctxd->lower.data = htole32(cmd | txd_lower | TSO_WORKAROUND);
                    // ctxd->upper.data = htole32(txd_upper);
                    // pidx_last = i;
                    // if (++i == scctx->isc_ntxd[0])
                    //     i = 0;
                    {
                        // limit borrow to this scope
                        let ctxd: &mut e1000_tx_desc = &mut txd_slice[i as usize];
                        // e1000_println!("2: did make ctxd i = {}", i);
                        ctxd.buffer_addr = seg_addr + seg_len;
                        ctxd.lower.data = cmd | txd_lower | TSO_WORKAROUND as u32;
                        ctxd.upper.data = txd_upper;
                    }
                    pidx_last = i;
                    i += 1;
                    if i == ntxd as u16 {
                        i = 0;
                    }

                    // DPRINTF(iflib_get_dev(sc->ctx), "TSO path pidx_last=%d i=%d ntxd[0]=%d\n",
                    //     pidx_last, i, scctx->isc_ntxd[0]);
                    e1000_println!(
                        "TSO path pidx_last={}, i={}, ntxd[0]={}",
                        pidx_last,
                        i,
                        ntxd
                    );
                // 	} else {
                } else {
                    // ctxd->buffer_addr = htole64(seg_addr);
                    // ctxd->lower.data = htole32(cmd | txd_lower | seg_len);
                    // ctxd->upper.data = htole32(txd_upper);
                    // pidx_last = i;
                    // if (++i == scctx->isc_ntxd[0])
                    //     i = 0;
                    // DPRINTF(iflib_get_dev(sc->ctx), "pidx_last=%d i=%d ntxd[0]=%d\n",
                    //     pidx_last, i, scctx->isc_ntxd[0]);
                    let ctxd: &mut e1000_tx_desc = &mut txd_slice[i as usize];
                    // e1000_println!("3: did make ctxd i = {}", i);
                    ctxd.buffer_addr = seg_addr;
                    ctxd.lower.data = cmd | txd_lower | seg_len as u32;
                    ctxd.upper.data = txd_upper;
                    pidx_last = i;
                    i += 1;
                    if i == ntxd as u16 {
                        i = 0;
                    }
                    e1000_println!("pidx_last={}, i={}, ntxd[0]={}", pidx_last, i, ntxd);
                    // }
                }
                // }
            }
            /*
             * Last Descriptor of Packet
             * needs End Of Packet (EOP)
             * and Report Status (RS)
             */
            e1000_println!("set eop pidx_last = {}", pidx_last);

            // ctxd->lower.data |= htole32(E1000_TXD_CMD_EOP | txd_flags);
            let ctxd: &mut e1000_tx_desc = &mut txd_slice[pidx_last as usize];
            unsafe {
                ctxd.lower.data |= E1000_TXD_CMD_EOP | txd_flags;
            }
        }
        // if (txd_flags) {
        // 	txr->tx_rsq[txr->tx_rs_pidx] = pidx_last;
        // 	DPRINTF(iflib_get_dev(sc->ctx), "setting to RS on %d rs_pidx %d first: %d\n",
        //          pidx_last, txr->tx_rs_pidx, first);
        // 	txr->tx_rs_pidx = (txr->tx_rs_pidx+1) & (ntxd-1);
        // 	MPASS(txr->tx_rs_pidx != txr->tx_rs_cidx);
        // }
        if txd_flags != 0 && nsegs != 0 {
            txr.tx_rsq[txr.tx_rs_pidx as usize] = pidx_last;
            e1000_println!(
                "Setting to RS on {} rs_pidx {} first: {}",
                pidx_last,
                txr.tx_rs_pidx,
                first
            );
            txr.tx_rs_pidx = (txr.tx_rs_pidx + 1) & (ntxd - 1) as u16;
            assert!(txr.tx_rs_pidx != txr.tx_rs_cidx);
        }
        // pi->ipi_new_pidx = i;
        pi.ipi_new_pidx = i;
        e1000_println!("done");

        // return (0);
        0
    }
    #[no_mangle]
    fn em_txd_flush(&mut self, txqid: u16, pidx: u16) {
        // log("txrx txd_flush\n\0");
        // e1000_println!();
        e1000_println!("txq: {}, pidx: {}", txqid, pidx);
        // struct adapter *adapter = arg;
        // struct em_tx_queue *que = &adapter->tx_queues[txqid];
        // struct tx_ring *txr = &que->txr;
        // E1000_WRITE_REG(&adapter->hw, E1000_TDT(txr->me), pidx);
        let me = self.tx_queues[txqid as usize].txr.me as usize;
        e1000_println!("txr.me = {}", me);
        do_write_register(self, E1000_TDT(me), pidx as u32);
        e1000_println!("done");
    }
    #[no_mangle]
    fn em_txd_credits_update(&mut self, txqid: u16, clear: bool) -> i32 {
        // log("txrx txd_credits_update\n\0");
        e1000_println!();
        // struct adapter *adapter = arg;
        // if_softc_ctx_t scctx = adapter->shared;
        // struct em_tx_queue *que = &adapter->tx_queues[txqid];
        // struct tx_ring *txr = &que->txr;
        let txq: &mut TxQueue = &mut self.tx_queues[txqid as usize];
        let txr: &mut TxRing = &mut txq.txr;

        // qidx_t processed = 0;
        // int updated;
        // qidx_t cur, prev, ntxd, rs_cidx;
        // int32_t delta;
        // uint8_t status;
        let mut processed: u16 = 0;
        let mut delta: i32;
        let mut status: u8;
        let mut cur: u16;
        let mut prev: u16;
        let mut rs_cidx: u16;
        let ntxd: u16;
        let updated: i32;

        // prev = txr->tx_cidx_processed;
        // ntxd = scctx->isc_ntxd[0];
        prev = txr.tx_cidx_processed;
        ntxd = self.iflib_shared.isc_ntxd[0] as u16;

        // rs_cidx = txr->tx_rs_cidx;
        // if (rs_cidx == txr->tx_rs_pidx)
        // 	return (0);
        // cur = txr->tx_rsq[rs_cidx];
        // MPASS(cur != QIDX_INVALID);
        // status = txr->tx_base[cur].upper.fields.status;
        // updated = !!(status & E1000_TXD_STAT_DD);
        rs_cidx = txr.tx_rs_cidx;
        if rs_cidx == txr.tx_rs_pidx {
            return 0;
        }
        cur = txr.tx_rsq[rs_cidx as usize];
        assert!(cur != QIDX_INVALID as u16);
        unsafe {
            let txd_slice: &[e1000_tx_desc] = txr.txd_tx_desc_slice(ntxd as usize);
            let txd: &e1000_tx_desc = &txd_slice[cur as usize];
            status = txd.upper.fields.status;
        }
        updated = match status & E1000_TXD_STAT_DD != 0 {
            true => 1,
            false => 0,
        };

        // if (clear == false || updated == 0)
        // 	return (updated);
        if clear == false || updated == 0 {
            return updated;
        }

        // do {
        // 	delta = (int32_t)cur - (int32_t)prev;
        // 	MPASS(prev == 0 || delta != 0);
        // 	if (delta < 0)
        // 		delta += ntxd;
        // 	DPRINTF(iflib_get_dev(adapter->ctx),
        // 		      "%s: cidx_processed=%u cur=%u clear=%d delta=%d\n",
        // 		      __FUNCTION__, prev, cur, clear, delta);

        // 	processed += delta;
        // 	prev  = cur;
        // 	rs_cidx = (rs_cidx + 1) & (ntxd-1);
        // 	if (rs_cidx  == txr->tx_rs_pidx)
        // 		break;
        // 	cur = txr->tx_rsq[rs_cidx];
        // 	MPASS(cur != QIDX_INVALID);
        // 	status = txr->tx_base[cur].upper.fields.status;
        // } while ((status & E1000_TXD_STAT_DD));
        loop {
            delta = cur as i32 - prev as i32;
            assert!(prev == 0 || delta != 0);
            if delta < 0 {
                delta += ntxd as i32;
            }
            e1000_println!(
                "cidx_processed = {}, cur = {}, clear = {}, delta = {}",
                prev,
                cur,
                clear,
                delta
            );
            processed += delta as u16;
            prev = cur;
            rs_cidx = (rs_cidx + 1) & (ntxd - 1);
            if rs_cidx == txr.tx_rs_pidx {
                break;
            }
            cur = txr.tx_rsq[rs_cidx as usize];
            assert!(cur != QIDX_INVALID as u16);

            let txd_slice: &[e1000_tx_desc] = txr.txd_tx_desc_slice(ntxd as usize);
            let txd: &e1000_tx_desc = &txd_slice[cur as usize];
            unsafe {
                status = txd.upper.fields.status;
            }
            if status & E1000_TXD_STAT_DD == 0 {
                break;
            }
        }

        // txr->tx_rs_cidx = rs_cidx;
        // txr->tx_cidx_processed = prev;
        // return(processed);
        txr.tx_rs_cidx = rs_cidx;
        txr.tx_cidx_processed = prev;
        processed as i32
    }
    fn em_rxd_available(&mut self, rxqid: u16, idx: u16, budget: u16) -> i32 {
        e1000_println!();
        // struct adapter *sc = arg;
        // if_softc_ctx_t scctx = sc->shared;
        // struct em_rx_queue *que = &sc->rx_queues[rxqid];
        // struct rx_ring *rxr = &que->rxr;
        // union e1000_rx_desc_extended *rxd;
        // u32 staterr = 0;
        // int cnt, i;
        let mut staterr: u32 = 0;
        let mut cnt: usize;
        let mut i: usize;

        let rxq: &mut RxQueue = &mut self.rx_queues[rxqid as usize];
        let rxr: &mut RxRing = &mut rxq.rxr;
        let nrxd = self.iflib_shared.isc_nrxd[0] as usize;
        let rxd_slice: &mut [e1000_rx_desc_extended] = rxr.rxd_rx_desc_extended_slice(nrxd);

        // unsafe {
        //     for i in 0..128 {
        //         let rxd = &rxd_slice[i];
        //         e1000_println!(
        //             "RXD: Addr: 0x{:x}. Stat: 0x{:x}. Len: {}",
        //             rxd.read.buffer_addr,
        //             rxd.wb.upper.status_error,
        //             rxd.wb.upper.length
        //         );
        //     }
        // }
        // if (budget == 1) {
        // 	rxd = &rxr->rx_base[idx];
        // 	staterr = le32toh(rxd->wb.upper.status_error);
        // 	return (staterr & E1000_RXD_STAT_DD);
        // }
        if budget == 1 {
            staterr = unsafe { le32toh!(rxd_slice[idx as usize].wb.upper.status_error) } as u32;
            e1000_println!(
                "Budget == 1, returning early {}",
                staterr & E1000_RXD_STAT_DD
            );
            return (staterr & E1000_RXD_STAT_DD) as i32;
        }

        // for (cnt = 0, i = idx; cnt < scctx->isc_nrxd[0] && cnt <= budget;) {
        // 	rxd = &rxr->rx_base[i];
        // 	staterr = le32toh(rxd->wb.upper.status_error);
        // 	if ((staterr & E1000_RXD_STAT_DD) == 0)
        // 		break;
        // 	if (++i == scctx->isc_nrxd[0]) {
        // 		i = 0;
        // 	}
        // 	if (staterr & E1000_RXD_STAT_EOP)
        // 		cnt++;
        // }
        // e1000_println!("Got budget {}", budget);
        i = idx as usize;
        cnt = 0;
        while cnt < nrxd && cnt <= budget as usize {
            staterr = unsafe { le32toh!(rxd_slice[i].wb.upper.status_error) } as u32;

            // e1000_println!("wb: {:?}", unsafe { rxd_slice[i].wb });

            e1000_println!("In loop got staterr 0x{:x}", staterr);
            if !btst!(staterr, E1000_RXD_STAT_DD) {
                break;
            }
            i += 1;
            if i == nrxd {
                i = 0;
            }
            if btst!(staterr, E1000_RXD_STAT_EOP) {
                cnt += 1;
            }
        }
        // return (cnt);
        // e1000_println!("Returning count = {}", cnt);
        cnt as i32
    }
    fn lem_rxd_available(&mut self, rxqid: u16, idx: u16, budget: u16) -> i32 {
        // log("txrx rxd_available\n\0");
        e1000_println!();
        // struct adapter *sc = arg;
        // if_softc_ctx_t scctx = sc->shared;
        // struct em_rx_queue *que = &sc->rx_queues[rxqid];
        // struct rx_ring *rxr = &que->rxr;
        // struct e1000_rx_desc *rxd;
        // u32 staterr = 0;
        // int cnt, i;
        let mut staterr: u32;
        let mut cnt: usize;
        let mut i: usize;

        let rxq: &mut RxQueue = &mut self.rx_queues[rxqid as usize];
        let rxr: &mut RxRing = &mut rxq.rxr;
        let nrxd = self.iflib_shared.isc_nrxd[0] as usize;
        let rxd_slice: &mut [e1000_rx_desc] = rxr.rxd_rx_desc_slice(nrxd);

        // if (budget == 1) {
        // 	rxd = (struct e1000_rx_desc *)&rxr->rx_base[idx];
        // 	staterr = rxd->status;
        // 	return (staterr & E1000_RXD_STAT_DD);
        // }
        if budget == 1 {
            staterr = rxd_slice[idx as usize].status as u32;
            return (staterr & E1000_RXD_STAT_DD) as i32;
        }

        // for (cnt = 0, i = idx; cnt < scctx->isc_nrxd[0] && cnt <= budget;) {
        // 	rxd = (struct e1000_rx_desc *)&rxr->rx_base[i];
        // 	staterr = rxd->status;

        // 	if ((staterr & E1000_RXD_STAT_DD) == 0)
        // 		break;

        // 	if (++i == scctx->isc_nrxd[0])
        // 		i = 0;

        // 	if (staterr & E1000_RXD_STAT_EOP)
        // 		cnt++;
        // }
        i = idx as usize;
        cnt = 0;
        while cnt < nrxd && cnt <= budget as usize {
            staterr = rxd_slice[i].status as u32;
            if staterr & E1000_RXD_STAT_DD == 0 {
                break;
            }
            i += 1;
            if i == nrxd {
                i = 0;
            }
            if staterr & E1000_RXD_STAT_EOP != 0 {
                cnt += 1;
            }
        }
        // return (cnt);

        // e1000_println!("finished");
        cnt as i32
    }
    fn em_rxd_pkt_get(&mut self, ri: &mut IfRxdInfo) -> i32 {
        // log("txrx rxd_pkt_get\n\0");
        e1000_println!();

        // struct adapter *adapter = arg;
        // if_softc_ctx_t scctx = adapter->shared;
        // struct em_rx_queue *que = &adapter->rx_queues[ri->iri_qsidx];
        // struct rx_ring *rxr = &que->rxr;
        // union e1000_rx_desc_extended *rxd;

        // u16 len;
        // u32 pkt_info;
        // u32 staterr = 0;
        // bool eop;
        // int i, cidx, vtag;
        let mut len: u16;
        let mut pkt_info: u32;
        let mut staterr: u32 = 0;
        let mut eop: bool;
        let mut i: usize = 0;
        let mut cidx: usize;
        let mut vtag: u16 = 0;

        // i = vtag = 0;
        // cidx = ri->iri_cidx;
        cidx = ri.iri_cidx as usize;

        let mut last_rxd: usize;

        {
            let rxq: &mut RxQueue = &mut self.rx_queues[ri.iri_qsidx as usize];
            let rxr: &mut RxRing = &mut rxq.rxr;
            let nrxd = self.iflib_shared.isc_nrxd[0] as usize;
            let rxd_slice: &mut [e1000_rx_desc_extended] = rxr.rxd_rx_desc_extended_slice(nrxd);

            // do {
            loop {
                // rxd = &rxr->rx_base[cidx];
                // staterr = le32toh(rxd->wb.upper.status_error);
                // pkt_info = le32toh(rxd->wb.lower.mrq);
                let rxd: &mut e1000_rx_desc_extended = &mut rxd_slice[cidx];
                last_rxd = cidx;
                staterr = unsafe { le32toh!(rxd.wb.upper.status_error) };
                pkt_info = unsafe { le32toh!(rxd.wb.lower.mrq) };

                // e1000_println!("{}: {:?}", cidx, rxd);

                /* Error Checking then decrement count */
                // MPASS ((status & E1000_RXD_STAT_DD) != 0);
                // e1000_println!("asserting: {} != 0", status & E1000_RXD_STAT_DD);
                // assert!(status & E1000_RXD_STAT_DD != 0);
                if staterr & E1000_RXD_STAT_DD == 0 {
                    eprintln!("E1000_RXD_STAT_DD Assert fail - return EBADMSG");
                    // EBADMSG 89 /* Bad message */
                    return 89;
                }

                // len = le16toh(rxd->wb.upper.length);
                // ri->iri_len += len;
                len = unsafe { le16toh!(rxd.wb.upper.length) };
                ri.iri_len += len;

                // eop = (staterr & E1000_RXD_STAT_EOP) != 0;
                eop = btst!(staterr, E1000_RXD_STAT_EOP);

                /* Make sure bad packets are discarded */
                // 	if (staterr & E1000_RXD_ERR_FRAME_ERR_MASK) {
                // 		adapter->dropped_pkts++;
                // 		return (EBADMSG);
                // 	}
                if btst!(staterr, E1000_RXDEXT_ERR_FRAME_ERR_MASK) {
                    self.dropped_pkts += 1;
                    eprintln!("E1000_RXD_ERR_FRAME_ERR_MASK set - return EBADMSG");
                    // EBADMSG 89 /* Bad message */
                    return 89;
                }

                {
                    // 	ri->iri_frags[i].irf_flid = 0;
                    // 	ri->iri_frags[i].irf_idx = cidx;
                    // 	ri->iri_frags[i].irf_len = len;
                    // e1000_println!("before slice (i = {}, nfrags = {})", i, ri.iri_nfrags);
                    let frags: &mut [if_rxd_frag] = ri.frags_slice(i + 1);
                    // e1000_println!("before slice index");
                    let frag: &mut if_rxd_frag = &mut frags[i];
                    // e1000_println!("after slice");
                    frag.irf_flid = 0;
                    frag.irf_idx = cidx as u16;
                    frag.irf_len = len;
                }
                /* Zero out the receive descriptors status. */
                // 	rxd->status = 0;
                unsafe {
                    rxd.wb.upper.status_error &= htole32!(!0xFF);
                }

                // 	if (++cidx == scctx->isc_nrxd[0])
                // 		cidx = 0;
                // 	i++;
                cidx += 1;
                if cidx == nrxd {
                    cidx = 0;
                }
                i += 1;

                // } while (!eop);
                if eop {
                    break;
                }
            } // End of loop

            // ri->iri_flowid = le32toh(rxd->wb.lower.hi_dword.rss);
            // ri->iri_rsstype = em_determine_rsstype(pkt_info);
            let rxd: &mut e1000_rx_desc_extended = &mut rxd_slice[last_rxd];
            ri.iri_flowid = unsafe { le32toh!(rxd.wb.lower.hi_dword.rss) };
            ri.iri_rsstype = em_determine_rsstype(pkt_info) as u8;

            // if (staterr & E1000_RXD_STAT_VP) {
            //     vtag = le16toh(rxd->wb.upper.vlan);
            // }
            if btst!(staterr, E1000_RXD_STAT_VP) {
                vtag = unsafe { le16toh!(rxd.wb.upper.vlan) };
            }
        } // End of rxq.rxd_slice borrow scope

        /* XXX add a faster way to look this up */
        // if (adapter->hw.mac.type >= e1000_82543)
        // 	em_receive_checksum(staterr, ri);
        if self.hw.mac.mac_type >= MacType::Mac_82543 {
            em_receive_checksum(staterr, ri);
        }

        ri.iri_vtag = vtag;
        if vtag != 0 {
            ri.iri_flags |= M_VLANTAG as u8;
        }

        ri.iri_nfrags = i as u8;
        0
    }
    fn lem_rxd_pkt_get(&mut self, ri: &mut IfRxdInfo) -> i32 {
        // log("txrx rxd_pkt_get\n\0");
        e1000_println!();

        // struct adapter *adapter = arg;
        // if_softc_ctx_t scctx = adapter->shared;
        // struct em_rx_queue *que = &adapter->rx_queues[ri->iri_qsidx];
        // struct rx_ring *rxr = &que->rxr;
        // struct e1000_rx_desc *rxd;
        // u16 len;
        // u32 status, errors;
        // bool eop;
        // int i, cidx;
        let mut len: u16;
        let mut status: u32;
        let mut errors: u32;
        let mut eop: bool;
        let mut i: usize = 0;
        let mut cidx: usize;

        // status = errors = i = 0;
        // cidx = ri->iri_cidx;
        cidx = ri.iri_cidx as usize;

        let mut last_rxd: usize;

        {
            let rxq: &mut RxQueue = &mut self.rx_queues[ri.iri_qsidx as usize];
            let rxr: &mut RxRing = &mut rxq.rxr;
            let nrxd = self.iflib_shared.isc_nrxd[0] as usize;
            let rxd_slice: &mut [e1000_rx_desc] = rxr.rxd_rx_desc_slice(nrxd);

            // do {
            loop {
                // 	rxd = (struct e1000_rx_desc *)&rxr->rx_base[cidx];
                // 	status = rxd->status;
                // 	errors = rxd->errors;
                let rxd: &mut e1000_rx_desc = &mut rxd_slice[cidx];
                last_rxd = cidx;

                e1000_println!("{}: {:?}", cidx, rxd);

                status = rxd.status as u32;
                errors = rxd.errors as u32;

                /* Error Checking then decrement count */
                // MPASS ((status & E1000_RXD_STAT_DD) != 0);
                e1000_println!("asserting: {} != 0", status & E1000_RXD_STAT_DD);
                if status & E1000_RXD_STAT_DD == 0 {
                    e1000_println!("assert fail");
                    // EBADMSG 89 /* Bad message */
                    return 89;
                }
                // assert!(status & E1000_RXD_STAT_DD != 0);

                // 	len = le16toh(rxd->length);
                // 	ri->iri_len += len;
                len = rxd.length;
                ri.iri_len += len;

                // 	eop = (status & E1000_RXD_STAT_EOP) != 0;
                eop = status & E1000_RXD_STAT_EOP != 0;

                /* Make sure bad packets are discarded */
                // 	if (errors & E1000_RXD_ERR_FRAME_ERR_MASK) {
                // 		adapter->dropped_pkts++;
                // 		/* XXX fixup if common */
                // 		return (EBADMSG);
                // 	}
                if errors & E1000_RXD_ERR_FRAME_ERR_MASK != 0 {
                    self.dropped_pkts += 1;
                    e1000_println!("bad packet");
                    // EBADMSG 89 /* Bad message */
                    return 89;
                }

                // Limit scope of ri borrow
                {
                    // 	ri->iri_frags[i].irf_flid = 0;
                    // 	ri->iri_frags[i].irf_idx = cidx;
                    // 	ri->iri_frags[i].irf_len = len;
                    e1000_println!("before slice (i = {}, nfrags = {})", i, ri.iri_nfrags);
                    let frags: &mut [if_rxd_frag] = ri.frags_slice(i + 1);
                    e1000_println!("before slice index");
                    let frag: &mut if_rxd_frag = &mut frags[i];
                    e1000_println!("after slice");
                    frag.irf_flid = 0;
                    frag.irf_idx = cidx as u16;
                    frag.irf_len = len;
                }

                /* Zero out the receive descriptors status. */
                // 	rxd->status = 0;
                rxd.status = 0;

                // 	if (++cidx == scctx->isc_nrxd[0])
                // 		cidx = 0;
                // 	i++;
                cidx += 1;
                if cidx == nrxd {
                    cidx = 0;
                }
                i += 1;

                // } while (!eop);
                if eop {
                    e1000_println!("breaking loop");
                    break;
                }
                e1000_println!("loop end - repeating");
            }
            e1000_println!("loop done");
            // if (status & E1000_RXD_STAT_VP) {
            // 	ri->iri_vtag = le16toh(rxd->special);
            // 	ri->iri_flags |= M_VLANTAG;
            // }
            if status & E1000_RXD_STAT_VP != 0 {
                let rxd: &mut e1000_rx_desc = &mut rxd_slice[last_rxd];
                ri.iri_vtag = rxd.special;
                ri.iri_flags |= M_VLANTAG as u8;
            }
        }
        /* XXX add a faster way to look this up */
        // if (adapter->hw.mac.type >= e1000_82543 && !(status & E1000_RXD_STAT_IXSM))
        // 	lem_receive_checksum(status, errors, ri);
        if self.hw.mac.mac_type >= MacType::Mac_82543 && status & E1000_RXD_STAT_IXSM == 0 {
            lem_receive_checksum(status, errors, ri);
        }

        // ri->iri_nfrags = i;
        ri.iri_nfrags = i as u8;

        // >>>>> never reach here!!

        e1000_println!("finished");

        // return (0);
        0
    }
    fn em_rxd_refill(&mut self, iru: &mut IfRxdUpdate) {
        e1000_println!();
        // struct adapter *sc = arg;
        // if_softc_ctx_t scctx = sc->shared;
        // uint16_t rxqid = iru->iru_qsidx;
        // struct em_rx_queue *que = &sc->rx_queues[rxqid];
        // struct rx_ring *rxr = &que->rxr;
        // union e1000_rx_desc_extended *rxd;
        // uint64_t *paddrs;
        // uint32_t next_pidx, pidx;
        // uint16_t count;
        // int i;

        // paddrs = iru->iru_paddrs;
        // pidx = iru->iru_pidx;
        // count = iru->iru_count;

        let qid: usize = iru.iru_qsidx as usize;
        let count: usize = iru.iru_count as usize;
        let pidx: usize = iru.iru_pidx as usize;
        let mut next_pidx = pidx;

        e1000_println!("Got queue id: {}", qid);
        e1000_println!("Got iru count: {}", count);
        e1000_println!("Got max nrxd: {}", self.iflib_shared.isc_nrxd[0]);
        e1000_println!("RX QUEUES count: {:?}", self.rx_queues.len());

        let nrxd: usize = self.iflib_shared.isc_nrxd[0] as usize;

        let paddrs: *mut u64 = iru.iru_paddrs;
        let paddrs_slice: &[u64] = unsafe { kernel::slice::from_raw_parts(paddrs, count) };

        // for (i = 0, next_pidx = pidx; i < count; i++) {
        //     rxd = &rxr->rx_base[next_pidx];
        //     rxd->read.buffer_addr = htole64(paddrs[i]);
        //     /* DD bits must be cleared */
        //     rxd->wb.upper.status_error = 0;

        //     if (++next_pidx == scctx->isc_nrxd[0])
        // 	next_pidx = 0;
        // }
        let rxq: &mut RxQueue = &mut self.rx_queues[qid];
        let rxr: &mut RxRing = &mut rxq.rxr;
        let rxd_slice: &mut [e1000_rx_desc_extended] = rxr.rxd_rx_desc_extended_slice(nrxd);

        // unsafe {
        //     for i in 0..count {
        //         let rxd = &rxd_slice[i];
        //         e1000_println!(
        //             "RXD: Addr: 0x{:x}. Stat: 0x{:x}. Len: {}",
        //             rxd.read.buffer_addr,
        //             rxd.wb.upper.status_error,
        //             rxd.wb.upper.length
        //         );
        //     }
        // }

        for i in 0..count {
            let rxd: &mut e1000_rx_desc_extended = &mut rxd_slice[next_pidx];

            unsafe {
                rxd.read.buffer_addr = htole64!(paddrs_slice[i]);
                rxd.wb.upper.status_error = 0;
            }

            // println!(
            //     "At index {} of {}. Buffer paddr: 0x{:x}",
            //     i,
            //     count - 1,
            //     htole64!(paddrs_slice[i])
            // );
            // println!("{:?}", rxd);

            next_pidx += 1;
            if next_pidx == nrxd as usize {
                next_pidx = 0;
            }
        }
    }
    fn lem_rxd_refill(&mut self, iru: &mut IfRxdUpdate) {
        // log("txrx rxd_refill\n\0");
        e1000_println!();
        // struct adapter *sc = arg;
        // if_softc_ctx_t scctx = sc->shared;
        // struct em_rx_queue *que = &sc->rx_queues[iru->iru_qsidx];
        // struct rx_ring *rxr = &que->rxr;
        // struct e1000_rx_desc *rxd;
        // uint64_t *paddrs;
        // uint32_t next_pidx, pidx;
        // uint16_t count;
        // int i;

        // paddrs = iru->iru_paddrs;
        // pidx = iru->iru_pidx;
        // count = iru->iru_count;

        let qid: usize = iru.iru_qsidx as usize;
        let count: usize = iru.iru_count as usize;
        let pidx: usize = iru.iru_pidx as usize;
        let mut next_pidx = pidx;

        e1000_println!("Got queue id: {}", qid);
        e1000_println!("Got iru count: {}", count);
        e1000_println!("Got max nrxd: {}", self.iflib_shared.isc_nrxd[0]);
        e1000_println!("RX QUEUES count: {:?}", self.rx_queues.len());

        let nrxd: usize = self.iflib_shared.isc_nrxd[0] as usize;

        let paddrs: *mut u64 = iru.iru_paddrs;
        let paddrs_slice: &[u64] = unsafe { kernel::slice::from_raw_parts(paddrs, count) };

        // for (i = 0, next_pidx = pidx; i < count; i++) {
        // 	rxd = (struct e1000_rx_desc *)&rxr->rx_base[next_pidx];
        // 	rxd->buffer_addr = htole64(paddrs[i]);
        // 	/* status bits must be cleared */
        // 	rxd->status = 0;

        // 	if (++next_pidx == scctx->isc_nrxd[0])
        // 		next_pidx = 0;
        // }

        let rxq: &mut RxQueue = &mut self.rx_queues[qid];
        let rxr: &mut RxRing = &mut rxq.rxr;
        let rxd_slice: &mut [e1000_rx_desc] = rxr.rxd_rx_desc_slice(nrxd);

        for i in 0..count {
            let rxd: &mut e1000_rx_desc = &mut rxd_slice[next_pidx];

            rxd.buffer_addr = paddrs_slice[i];
            rxd.status = 0;

            // println!(
            //     "At index {} of {}. Buffer paddr: 0x{:x}",
            //     i,
            //     count - 1,
            //     rxd.buffer_addr
            // );
            // println!("{:?}", rxd);

            next_pidx += 1;
            if next_pidx == nrxd as usize {
                next_pidx = 0;
            }
        }
        e1000_println!("finished");
    }
    fn em_rxd_flush(&mut self, rxqid: u16, flid: u8, pidx: u16) {
        // log("txrx rxd_flush\n\0");
        e1000_println!();
        // struct adapter *sc = arg;
        // struct em_rx_queue *que = &sc->rx_queues[rxqid];
        // struct rx_ring *rxr = &que->rxr;
        // E1000_WRITE_REG(&sc->hw, E1000_RDT(rxr->me), pidx);
        let rxr: &RxRing = &self.rx_queues[rxqid as usize].rxr;
        self.write_register(E1000_RDT(rxr.me as usize), pidx as u32);
    }
    fn em_intr(&mut self) -> i32 {
        // unsafe {
        // kernel::sys::systm_sys::uprintf("txrx::intr() begin".as_ptr() as *const i8);
        // kernel::sys::systm_sys::printf("txrx::intr() begin".as_ptr() as *const i8);
        // kernel::sys::systm_sys::log(0, "txrx::intr() begin".as_ptr() as *const i8);
        // }
        // printf("txrx intr\n\0");
        // e1000_println!();
        // 	struct adapter *adapter = arg;
        // 	iflib_t ctx = adapter->ctx;
        // 	u32 reg_icr;

        // 	reg_icr = E1000_READ_REG(&adapter->hw, E1000_ICR);
        let reg_icr: u32 = do_read_register(self, E1000_ICR);

        // 	if (adapter->intr_type != IFLIB_INTR_LEGACY)
        // 		goto skip_stray;
        if self.iflib_shared.isc_intr == iflib_intr_mode_t::IFLIB_INTR_LEGACY {
            // 	/* Hot eject? */
            // 	if (reg_icr == 0xffffffff)
            // 		return FILTER_STRAY;
            if reg_icr == 0xffffffff {
                // unsafe {
                //     kernel::sys::systm_sys::uprintf("txrx::intr() stray".as_ptr() as *const i8);
                //     kernel::sys::systm_sys::printf("txrx::intr() stray".as_ptr() as *const i8);
                // }
                return FILTER_STRAY as i32;
            }

            // 	/* Definitely not our interrupt. */
            // 	if (reg_icr == 0x0)
            // 		return FILTER_STRAY;
            if reg_icr == 0x0 {
                // unsafe {
                //     kernel::sys::systm_sys::uprintf("txrx::intr() 0x0".as_ptr() as *const i8);
                //     kernel::sys::systm_sys::printf("txrx::intr() 0x0".as_ptr() as *const i8);
                // }
                return FILTER_STRAY as i32;
            }

            // 	/*
            // 	 * Starting with the 82571 chip, bit 31 should be used to
            // 	 * determine whether the interrupt belongs to us.
            // 	 */
            // 	if (adapter->hw.mac.type >= e1000_82571 &&
            // 	    (reg_icr & E1000_ICR_INT_ASSERTED) == 0)
            // 		return FILTER_STRAY;
            if self.hw.mac.mac_type >= MacType::Mac_82571 && !btst!(reg_icr, E1000_ICR_INT_ASSERTED)
            {
                return FILTER_STRAY as i32;
            }
        }
        // unsafe {
        // kernel::sys::systm_sys::uprintf("txrx::intr() OK".as_ptr() as *const i8);
        // kernel::sys::systm_sys::printf("txrx::intr() OK".as_ptr() as *const i8);
        // }
        // skip_stray:
        // 	/* Link status change */
        // 	if (reg_icr & (E1000_ICR_RXSEQ | E1000_ICR_LSC)) {
        // 		adapter->hw.mac.get_link_status = 1;
        // 		iflib_admin_intr_deferred(ctx);
        // 	}
        if btst!(reg_icr, E1000_ICR_RXSEQ | E1000_ICR_LSC) {
            self.hw.mac.get_link_status = true;
            self.iflib.admin_intr_deferred();
        }
        // 	if (reg_icr & E1000_ICR_RXO)
        // 		adapter->rx_overruns++;
        if btst!(reg_icr, E1000_ICR_RXO) {
            self.rx_overruns += 1;
        }

        // unsafe {
        //     kernel::sys::systm_sys::log(10, "txrx::intr() OK".as_ptr() as *const i8);
        // }

        // 	return (FILTER_SCHEDULE_THREAD);
        FILTER_SCHEDULE_THREAD as i32
    }
}

/*********************************************************************
 *
 *  Verify that the hardware indicated that the checksum is valid.
 *  Inform the stack about the status of checksum so that stack
 *  doesn't spend time verifying the checksum.
 *
 *********************************************************************/
pub fn lem_receive_checksum(status: u32, errors: u32, ri: &mut IfRxdInfo) {
    e1000_println!();
    // /* Did it pass? */
    // if (status & E1000_RXD_STAT_IPCS && !(errors & E1000_RXD_ERR_IPE))
    //     ri->iri_csum_flags = (CSUM_IP_CHECKED|CSUM_IP_VALID);
    if status & E1000_RXD_STAT_IPCS != 0 && errors & E1000_RXD_ERR_IPE == 0 {
        ri.iri_csum_flags = CSUM_IP_CHECKED | CSUM_IP_VALID;
    }

    // if (status & E1000_RXD_STAT_TCPCS) {
    //     /* Did it pass? */
    //     if (!(errors & E1000_RXD_ERR_TCPE)) {
    // 	ri->iri_csum_flags |=
    // 	    (CSUM_DATA_VALID | CSUM_PSEUDO_HDR);
    // 	ri->iri_csum_data = htons(0xffff);
    //     }
    // }
    if status & E1000_RXD_STAT_TCPCS != 0 {
        if errors & E1000_RXD_ERR_TCPE == 0 {
            ri.iri_csum_flags |= CSUM_DATA_VALID | CSUM_PSEUDO_HDR;
            ri.iri_csum_data = htons!(0xffff) as u32;
        }
    }
}

/********************************************************************
 *
 *  Parse the packet type to determine the appropriate hash
 *
 ******************************************************************/
pub fn em_determine_rsstype(pkt_info: u32) -> u32 {
    match pkt_info & E1000_RXDADV_RSSTYPE_MASK {
        E1000_RXDADV_RSSTYPE_IPV4_TCP => M_HASHTYPE_RSS_TCP_IPV4,
        E1000_RXDADV_RSSTYPE_IPV4 => M_HASHTYPE_RSS_IPV4,
        E1000_RXDADV_RSSTYPE_IPV6_TCP => M_HASHTYPE_RSS_TCP_IPV6,
        E1000_RXDADV_RSSTYPE_IPV6_EX => M_HASHTYPE_RSS_IPV6_EX,
        E1000_RXDADV_RSSTYPE_IPV6 => M_HASHTYPE_RSS_IPV6,
        E1000_RXDADV_RSSTYPE_IPV6_TCP_EX => M_HASHTYPE_RSS_TCP_IPV6_EX,
        _ => M_HASHTYPE_OPAQUE,
    }
}

pub fn em_receive_checksum(status: u32, ri: &mut IfRxdInfo) {
    ri.iri_csum_flags = 0;

    /* Ignore Checksum bit is set */
    if btst!(status, E1000_RXD_STAT_IXSM) {
        return;
    }

    /* If the IP checksum exists and there is no IP Checksum error */
    if (status & (E1000_RXD_STAT_IPCS | E1000_RXDEXT_STATERR_IPE)) == E1000_RXD_STAT_IPCS {
        ri.iri_csum_flags = CSUM_IP_CHECKED | CSUM_IP_VALID;
    }

    /* TCP or UDP checksum */
    if (status & (E1000_RXD_STAT_TCPCS | E1000_RXDEXT_STATERR_TCPE)) == E1000_RXD_STAT_TCPCS {
        ri.iri_csum_flags |= CSUM_DATA_VALID | CSUM_PSEUDO_HDR;
        ri.iri_csum_data = htons!(0xffff) as u32;
    }
    if btst!(status, E1000_RXD_STAT_UDPCS) {
        ri.iri_csum_flags |= CSUM_DATA_VALID | CSUM_PSEUDO_HDR;
        ri.iri_csum_data = htons!(0xffff) as u32;
    }
}
