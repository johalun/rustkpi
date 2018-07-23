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

const MAX_INTS_PER_SEC: u32 = 8000;
const DEFAULT_ITR: u32 = 1000000000 / (MAX_INTS_PER_SEC * 256);

const TSO_WORKAROUND: u64 = 4;

impl Adapter {
    pub fn tso_setup(
        &mut self,
        pi: &IfPacketInfo,
        txd_upper: &mut u32,
        txd_lower: &mut u32,
    ) -> u16 {
        e1000_println!();

        let txq: &mut TxQueue = &mut self.tx_queues[pi.ipi_qsidx as usize];
        let txr: &mut TxRing = &mut txq.txr;

        let hdr_len = pi.ipi_ehdrlen + pi.ipi_ip_hlen + pi.ipi_tcp_hlen;
        *txd_lower = E1000_TXD_CMD_DEXT | E1000_TXD_DTYP_D | E1000_TXD_CMD_TSE;

        /* IP and/or TCP header checksum calculation and insertion. */
        *txd_upper = (E1000_TXD_POPTS_IXSM | E1000_TXD_POPTS_TXSM) << 8;

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
            unsafe {
                txd.tcp_seg_setup.fields.mss = pi.ipi_tso_segsz;
                txd.tcp_seg_setup.fields.hdr_len = hdr_len;
            }

            txd.cmd_and_length = self.txd_cmd |
            E1000_TXD_CMD_DEXT |	/* Extended descr */
            E1000_TXD_CMD_TSE |	/* TSE context */
            E1000_TXD_CMD_IP |	/* Do IP csum */
            E1000_TXD_CMD_TCP |	/* Do TCP checksum */
            (pi.ipi_len - hdr_len as u32); /* Total len */

            cur += 1;
            if cur == self.iflib_shared.isc_ntxd[0] as u16 {
                cur = 0;
            }
        }
        txr.tx_tso = true;

        cur
    }

    pub fn transmit_checksum_setup(
        &mut self,
        pi: &IfPacketInfo,
        txd_upper: &mut u32,
        txd_lower: &mut u32,
    ) -> u16 {
        e1000_println!();

        let mut cur;
        let hdr_len;
        let mut cmd;
        let csum_flags = pi.ipi_csum_flags;

        let txq: &mut TxQueue = &mut self.tx_queues[pi.ipi_qsidx as usize];
        let txr: &mut TxRing = &mut txq.txr;

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
        const DONT_FORCE_CTX: bool = true;

        if DONT_FORCE_CTX && self.iflib_shared.isc_ntxqsets == 1
            && txr.csum_lhlen == pi.ipi_ehdrlen as i32
            && txr.csum_iphlen == pi.ipi_ip_hlen as i32
            && txr.csum_flags == csum_flags as i32
        {
            *txd_upper = txr.csum_txd_upper;
            *txd_lower = txr.csum_txd_lower;
            return cur;
        }
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
            txd.tcp_seg_setup.data = 0;
            txd.cmd_and_length = E1000_TXD_CMD_IFCS | E1000_TXD_CMD_DEXT | cmd;
        }
        txr.csum_lhlen = pi.ipi_ehdrlen as i32;
        txr.csum_iphlen = pi.ipi_ip_hlen as i32;
        txr.csum_flags = csum_flags as i32;
        txr.csum_txd_upper = *txd_upper;
        txr.csum_txd_lower = *txd_lower;

        cur += 1;
        if cur == self.iflib_shared.isc_ntxd[0] as u16 {
            cur = 0;
        }
        cur
    }

    pub fn rx_queue_intr_enable(&mut self, rxqid: u16) {
        e1000_println!();

        let eims = self.rx_queues[rxqid as usize].eims;
        do_write_register(self, E1000_IMS, eims);
    }

    pub fn tx_queue_intr_enable(&mut self, txqid: u16) {
        e1000_println!();

        let eims = self.tx_queues[txqid as usize].eims;
        do_write_register(self, E1000_IMS, eims);
    }
}

impl IfTxRx for Adapter {
    fn em_txd_encap(&mut self, pi: &mut IfPacketInfo) -> i32 {
        // e1000_println!();

        let nsegs = pi.ipi_nsegs as usize;
        let segs: &mut [bus_dma_segment] =
            unsafe { kernel::slice::from_raw_parts_mut(pi.ipi_segs, nsegs as usize) };
        let csum_flags: u32 = pi.ipi_csum_flags;

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
            if !do_tso && txr.tx_tso {
                if nsegs == 1 {
                    tso_desc = true;
                }
                txr.tx_tso = false;
            }
        }

        /* Do hardware assists */
        if do_tso {
            i = self.tso_setup(pi, &mut txd_upper, &mut txd_lower);
            tso_desc = true;
        } else if csum_flags & EM_CSUM_OFFLOAD != 0 {
            i = self.transmit_checksum_setup(pi, &mut txd_upper, &mut txd_lower);
        }

        if pi.ipi_mflags & M_VLANTAG as u8 != 0 {
            txd_upper |= (pi.ipi_vtag as u32) << 16;
            txd_lower |= E1000_TXD_CMD_VLE;
        }

        let txq: &mut TxQueue = &mut self.tx_queues[pi.ipi_qsidx as usize];
        let txr: &mut TxRing = &mut txq.txr;

        let mut pidx_last = first;
        {
            // limit borrow of txr to this scope
            let txd_slice: &mut [e1000_tx_desc] = txr.txd_tx_desc_slice(ntxd as usize);

            for j in 0..nsegs {
                let seg_addr = segs[j].ds_addr;
                let mut seg_len = segs[j].ds_len;
                let cmd = E1000_TXD_CMD_IFCS | self.txd_cmd;

                /*
                 * TSO Workaround:
                 * If this is the last descriptor, we want to
                 * split it so we have a small final sentinel
                 */
                if tso_desc && (j == (nsegs - 1)) && (seg_len > 8) {
                    {
                        // limit borrow to this scope
                        let ctxd: &mut e1000_tx_desc = &mut txd_slice[i as usize];
                        seg_len -= TSO_WORKAROUND;
                        ctxd.buffer_addr = seg_addr;
                        ctxd.lower.data = cmd | txd_lower | seg_len as u32;
                        ctxd.upper.data = txd_upper;
                    }
                    i += 1;
                    if i == ntxd as u16 {
                        i = 0;
                    }

                    /* Now make the sentinel */
                    {
                        // limit borrow to this scope
                        let ctxd: &mut e1000_tx_desc = &mut txd_slice[i as usize];
                        ctxd.buffer_addr = seg_addr + seg_len;
                        ctxd.lower.data = cmd | txd_lower | TSO_WORKAROUND as u32;
                        ctxd.upper.data = txd_upper;
                    }
                    pidx_last = i;
                    i += 1;
                    if i == ntxd as u16 {
                        i = 0;
                    }
                } else {
                    let ctxd: &mut e1000_tx_desc = &mut txd_slice[i as usize];
                    ctxd.buffer_addr = seg_addr;
                    ctxd.lower.data = cmd | txd_lower | seg_len as u32;
                    ctxd.upper.data = txd_upper;
                    pidx_last = i;
                    i += 1;
                    if i == ntxd as u16 {
                        i = 0;
                    }
                }
            }
            /*
             * Last Descriptor of Packet
             * needs End Of Packet (EOP)
             * and Report Status (RS)
             */
            let ctxd: &mut e1000_tx_desc = &mut txd_slice[pidx_last as usize];
            unsafe {
                ctxd.lower.data |= E1000_TXD_CMD_EOP | txd_flags;
            }
        }
        if txd_flags != 0 && nsegs != 0 {
            txr.tx_rsq[txr.tx_rs_pidx as usize] = pidx_last;
            txr.tx_rs_pidx = (txr.tx_rs_pidx + 1) & (ntxd - 1) as u16;
            assert!(txr.tx_rs_pidx != txr.tx_rs_cidx);
        }
        pi.ipi_new_pidx = i;
        0
    }

    fn em_txd_flush(&mut self, txqid: u16, pidx: u16) {
        // e1000_println!();

        let me = self.tx_queues[txqid as usize].txr.me as usize;
        do_write_register(self, E1000_TDT(me), pidx as u32);
    }

    fn em_txd_credits_update(&mut self, txqid: u16, clear: bool) -> i32 {
        // e1000_println!();

        let txq: &mut TxQueue = &mut self.tx_queues[txqid as usize];
        let txr: &mut TxRing = &mut txq.txr;

        let mut processed: u16 = 0;
        let mut delta: i32;
        let mut status: u8;
        let mut cur: u16;
        let mut prev: u16;
        let mut rs_cidx: u16;
        let ntxd: u16;
        let updated: i32;

        prev = txr.tx_cidx_processed;
        ntxd = self.iflib_shared.isc_ntxd[0] as u16;

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

        if clear == false || updated == 0 {
            return updated;
        }

        loop {
            delta = cur as i32 - prev as i32;
            assert!(prev == 0 || delta != 0);
            if delta < 0 {
                delta += ntxd as i32;
            }
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

        txr.tx_rs_cidx = rs_cidx;
        txr.tx_cidx_processed = prev;
        processed as i32
    }

    fn em_rxd_available(&mut self, rxqid: u16, idx: u16, budget: u16) -> i32 {
        // e1000_println!();

        let mut staterr: u32 = 0;
        let mut cnt: usize;
        let mut i: usize;

        let rxq: &mut RxQueue = &mut self.rx_queues[rxqid as usize];
        let rxr: &mut RxRing = &mut rxq.rxr;
        let nrxd = self.iflib_shared.isc_nrxd[0] as usize;
        let rxd_slice: &mut [e1000_rx_desc_extended] = rxr.rxd_rx_desc_extended_slice(nrxd);

        if budget == 1 {
            staterr = unsafe { le32toh!(rxd_slice[idx as usize].wb.upper.status_error) } as u32;
            return (staterr & E1000_RXD_STAT_DD) as i32;
        }

        i = idx as usize;
        cnt = 0;
        while cnt < nrxd && cnt <= budget as usize {
            staterr = unsafe { le32toh!(rxd_slice[i].wb.upper.status_error) } as u32;

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
        cnt as i32
    }

    fn lem_rxd_available(&mut self, rxqid: u16, idx: u16, budget: u16) -> i32 {
        // e1000_println!();

        let mut staterr: u32;
        let mut cnt: usize;
        let mut i: usize;

        let rxq: &mut RxQueue = &mut self.rx_queues[rxqid as usize];
        let rxr: &mut RxRing = &mut rxq.rxr;
        let nrxd = self.iflib_shared.isc_nrxd[0] as usize;
        let rxd_slice: &mut [e1000_rx_desc] = rxr.rxd_rx_desc_slice(nrxd);

        if budget == 1 {
            staterr = rxd_slice[idx as usize].status as u32;
            return (staterr & E1000_RXD_STAT_DD) as i32;
        }

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
        cnt as i32
    }

    fn em_rxd_pkt_get(&mut self, ri: &mut IfRxdInfo) -> i32 {
        // e1000_println!();

        let mut len: u16;
        let mut pkt_info: u32;
        let mut staterr: u32 = 0;
        let mut eop: bool;
        let mut i: usize = 0;
        let mut cidx: usize;
        let mut vtag: u16 = 0;

        cidx = ri.iri_cidx as usize;
        let mut last_rxd: usize;
        {
            let rxq: &mut RxQueue = &mut self.rx_queues[ri.iri_qsidx as usize];
            let rxr: &mut RxRing = &mut rxq.rxr;
            let nrxd = self.iflib_shared.isc_nrxd[0] as usize;
            let rxd_slice: &mut [e1000_rx_desc_extended] = rxr.rxd_rx_desc_extended_slice(nrxd);

            loop {
                let rxd: &mut e1000_rx_desc_extended = &mut rxd_slice[cidx];
                last_rxd = cidx;
                staterr = unsafe { le32toh!(rxd.wb.upper.status_error) };
                pkt_info = unsafe { le32toh!(rxd.wb.lower.mrq) };

                /* Error Checking then decrement count */
                if staterr & E1000_RXD_STAT_DD == 0 {
                    eprintln!("E1000_RXD_STAT_DD Assert fail - return EBADMSG");
                    // EBADMSG 89 /* Bad message */
                    return 89;
                }

                len = unsafe { le16toh!(rxd.wb.upper.length) };
                ri.iri_len += len;

                eop = btst!(staterr, E1000_RXD_STAT_EOP);

                /* Make sure bad packets are discarded */
                if btst!(staterr, E1000_RXDEXT_ERR_FRAME_ERR_MASK) {
                    self.dropped_pkts += 1;
                    eprintln!("E1000_RXD_ERR_FRAME_ERR_MASK set - return EBADMSG");
                    // EBADMSG 89 /* Bad message */
                    return 89;
                }

                {
                    let frags: &mut [if_rxd_frag] = ri.frags_slice(i + 1);
                    let frag: &mut if_rxd_frag = &mut frags[i];
                    frag.irf_flid = 0;
                    frag.irf_idx = cidx as u16;
                    frag.irf_len = len;
                }
                /* Zero out the receive descriptors status. */
                unsafe {
                    rxd.wb.upper.status_error &= htole32!(!0xFF);
                }

                cidx += 1;
                if cidx == nrxd {
                    cidx = 0;
                }
                i += 1;

                if eop {
                    break;
                }
            } // End of loop

            let rxd: &mut e1000_rx_desc_extended = &mut rxd_slice[last_rxd];
            ri.iri_flowid = unsafe { le32toh!(rxd.wb.lower.hi_dword.rss) };
            ri.iri_rsstype = em_determine_rsstype(pkt_info) as u8;

            if btst!(staterr, E1000_RXD_STAT_VP) {
                vtag = unsafe { le16toh!(rxd.wb.upper.vlan) };
            }
        } // End of rxq.rxd_slice borrow scope

        /* XXX add a faster way to look this up */
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
        // e1000_println!();

        let mut len: u16;
        let mut status: u32;
        let mut errors: u32;
        let mut eop: bool;
        let mut i: usize = 0;
        let mut cidx: usize;

        cidx = ri.iri_cidx as usize;
        let mut last_rxd: usize;
        {
            let rxq: &mut RxQueue = &mut self.rx_queues[ri.iri_qsidx as usize];
            let rxr: &mut RxRing = &mut rxq.rxr;
            let nrxd = self.iflib_shared.isc_nrxd[0] as usize;
            let rxd_slice: &mut [e1000_rx_desc] = rxr.rxd_rx_desc_slice(nrxd);

            loop {
                let rxd: &mut e1000_rx_desc = &mut rxd_slice[cidx];
                last_rxd = cidx;

                status = rxd.status as u32;
                errors = rxd.errors as u32;

                /* Error Checking then decrement count */
                if status & E1000_RXD_STAT_DD == 0 {
                    eprintln!("bad packet");
                    // EBADMSG 89 /* Bad message */
                    return 89;
                }

                len = rxd.length;
                ri.iri_len += len;

                eop = status & E1000_RXD_STAT_EOP != 0;

                /* Make sure bad packets are discarded */
                if errors & E1000_RXD_ERR_FRAME_ERR_MASK != 0 {
                    self.dropped_pkts += 1;
                    eprintln!("bad packet");
                    // EBADMSG 89 /* Bad message */
                    return 89;
                }

                // Limit scope of ri borrow
                {
                    let frags: &mut [if_rxd_frag] = ri.frags_slice(i + 1);
                    let frag: &mut if_rxd_frag = &mut frags[i];
                    frag.irf_flid = 0;
                    frag.irf_idx = cidx as u16;
                    frag.irf_len = len;
                }

                /* Zero out the receive descriptors status. */
                rxd.status = 0;

                cidx += 1;
                if cidx == nrxd {
                    cidx = 0;
                }
                i += 1;

                if eop {
                    break;
                }
            }
            if status & E1000_RXD_STAT_VP != 0 {
                let rxd: &mut e1000_rx_desc = &mut rxd_slice[last_rxd];
                ri.iri_vtag = rxd.special;
                ri.iri_flags |= M_VLANTAG as u8;
            }
        }
        /* XXX add a faster way to look this up */
        if self.hw.mac.mac_type >= MacType::Mac_82543 && status & E1000_RXD_STAT_IXSM == 0 {
            lem_receive_checksum(status, errors, ri);
        }

        ri.iri_nfrags = i as u8;
        0
    }

    fn em_rxd_refill(&mut self, iru: &mut IfRxdUpdate) {
        // e1000_println!();

        let qid: usize = iru.iru_qsidx as usize;
        let count: usize = iru.iru_count as usize;
        let pidx: usize = iru.iru_pidx as usize;
        let mut next_pidx = pidx;

        let nrxd: usize = self.iflib_shared.isc_nrxd[0] as usize;

        let paddrs: *mut u64 = iru.iru_paddrs;
        let paddrs_slice: &[u64] = unsafe { kernel::slice::from_raw_parts(paddrs, count) };

        let rxq: &mut RxQueue = &mut self.rx_queues[qid];
        let rxr: &mut RxRing = &mut rxq.rxr;
        let rxd_slice: &mut [e1000_rx_desc_extended] = rxr.rxd_rx_desc_extended_slice(nrxd);

        for i in 0..count {
            let rxd: &mut e1000_rx_desc_extended = &mut rxd_slice[next_pidx];

            unsafe {
                rxd.read.buffer_addr = htole64!(paddrs_slice[i]);
                rxd.wb.upper.status_error = 0;
            }

            next_pidx += 1;
            if next_pidx == nrxd as usize {
                next_pidx = 0;
            }
        }
    }

    fn lem_rxd_refill(&mut self, iru: &mut IfRxdUpdate) {
        // e1000_println!();

        let qid: usize = iru.iru_qsidx as usize;
        let count: usize = iru.iru_count as usize;
        let pidx: usize = iru.iru_pidx as usize;
        let mut next_pidx = pidx;

        let nrxd: usize = self.iflib_shared.isc_nrxd[0] as usize;

        let paddrs: *mut u64 = iru.iru_paddrs;
        let paddrs_slice: &[u64] = unsafe { kernel::slice::from_raw_parts(paddrs, count) };

        let rxq: &mut RxQueue = &mut self.rx_queues[qid];
        let rxr: &mut RxRing = &mut rxq.rxr;
        let rxd_slice: &mut [e1000_rx_desc] = rxr.rxd_rx_desc_slice(nrxd);

        for i in 0..count {
            let rxd: &mut e1000_rx_desc = &mut rxd_slice[next_pidx];

            rxd.buffer_addr = paddrs_slice[i];
            rxd.status = 0;

            next_pidx += 1;
            if next_pidx == nrxd as usize {
                next_pidx = 0;
            }
        }
    }

    fn em_rxd_flush(&mut self, rxqid: u16, flid: u8, pidx: u16) {
        // e1000_println!();

        let rxr: &RxRing = &self.rx_queues[rxqid as usize].rxr;
        self.write_register(E1000_RDT(rxr.me as usize), pidx as u32);
    }

    fn em_intr(&mut self) -> i32 {
        let reg_icr: u32 = do_read_register(self, E1000_ICR);

        if self.iflib_shared.isc_intr == iflib_intr_mode_t::IFLIB_INTR_LEGACY {
            /* Hot eject? */
            if reg_icr == 0xffffffff {
                return FILTER_STRAY as i32;
            }

            /* Definitely not our interrupt. */
            if reg_icr == 0x0 {
                return FILTER_STRAY as i32;
            }

            /*
             * Starting with the 82571 chip, bit 31 should be used to
             * determine whether the interrupt belongs to us.
             */
            if self.hw.mac.mac_type >= MacType::Mac_82571 && !btst!(reg_icr, E1000_ICR_INT_ASSERTED)
            {
                return FILTER_STRAY as i32;
            }
        }

        if btst!(reg_icr, E1000_ICR_RXSEQ | E1000_ICR_LSC) {
            self.hw.mac.get_link_status = true;
            self.iflib.admin_intr_deferred();
        }

        if btst!(reg_icr, E1000_ICR_RXO) {
            self.rx_overruns += 1;
        }

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
    // e1000_println!();

    /* Did it pass? */
    if status & E1000_RXD_STAT_IPCS != 0 && errors & E1000_RXD_ERR_IPE == 0 {
        ri.iri_csum_flags = CSUM_IP_CHECKED | CSUM_IP_VALID;
    }

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
    // e1000_println!();

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
