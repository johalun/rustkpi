
/*
 * Do not use e1000_sys.rs directly. Copy over used bits to this
 * file instead.
 */

use kernel;
use kernel::prelude::v1::*;

use kernel::sys::iflib_sys::bool_;
use kernel::sys::iflib_sys::qidx_t;
use kernel::sys::iflib_sys::if_rxd_update;
use kernel::sys::iflib_sys::if_pkt_info_t;
use kernel::sys::iflib_sys::if_rxd_info_t;

use super::iflib::IfIrq;
// pub use super::e1000_sys::*;

// pub use super::e1000_sys::e1000_mac_operations;
// pub use super::e1000_sys::e1000_phy_operations;
// pub use super::e1000_sys::e1000_nvm_operations;
// pub use super::e1000_sys::e1000_mbx_operations;

pub use super::e1000_consts::*;

use e1000_osdep::Resource;




/* ICH GbE Flash Hardware Sequencing Flash Status Register bit breakdown */
/* Offset 04h HSFSTS */
#[repr(C)]
#[derive(Copy)]
pub union Ich8HwsFlashStatus {
    pub hsf_status: Ich8HsfStatus,
    pub regval: u16,
    _bindgen_union_align: u16,
}
impl Clone for Ich8HwsFlashStatus {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for Ich8HwsFlashStatus {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
impl ::kernel::fmt::Debug for Ich8HwsFlashStatus {
    fn fmt(&self, f: &mut ::kernel::fmt::Formatter) -> ::kernel::fmt::Result {
        write!(f, "Ich8HwsFlashStatus {{ union }}")
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct Ich8HsfStatus {
    pub _bitfield_1: [u8; 2usize],
    pub __bindgen_align: [u16; 0usize],
}
impl Clone for Ich8HsfStatus {
    fn clone(&self) -> Self {
        *self
    }
}
impl Ich8HsfStatus {
    #[inline]
    pub fn flcdone(&self) -> u16 {
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        let mask = 1u64 as u16;
        let val = (unit_field_val & mask) >> 0usize;
        unsafe { ::core::mem::transmute(val as u16) }
    }
    #[inline]
    pub fn set_flcdone(&mut self, val: u16) {
        let mask = 1u64 as u16;
        let val = val as u16 as u16;
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 0usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u16>(),
            );
        }
    }
    #[inline]
    pub fn flcerr(&self) -> u16 {
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        let mask = 2u64 as u16;
        let val = (unit_field_val & mask) >> 1usize;
        unsafe { ::core::mem::transmute(val as u16) }
    }
    #[inline]
    pub fn set_flcerr(&mut self, val: u16) {
        let mask = 2u64 as u16;
        let val = val as u16 as u16;
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 1usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u16>(),
            );
        }
    }
    #[inline]
    pub fn dael(&self) -> u16 {
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        let mask = 4u64 as u16;
        let val = (unit_field_val & mask) >> 2usize;
        unsafe { ::core::mem::transmute(val as u16) }
    }
    #[inline]
    pub fn set_dael(&mut self, val: u16) {
        let mask = 4u64 as u16;
        let val = val as u16 as u16;
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 2usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u16>(),
            );
        }
    }
    #[inline]
    pub fn berasesz(&self) -> u16 {
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        let mask = 24u64 as u16;
        let val = (unit_field_val & mask) >> 3usize;
        unsafe { ::core::mem::transmute(val as u16) }
    }
    #[inline]
    pub fn set_berasesz(&mut self, val: u16) {
        let mask = 24u64 as u16;
        let val = val as u16 as u16;
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 3usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u16>(),
            );
        }
    }
    #[inline]
    pub fn flcinprog(&self) -> u16 {
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        let mask = 32u64 as u16;
        let val = (unit_field_val & mask) >> 5usize;
        unsafe { ::core::mem::transmute(val as u16) }
    }
    #[inline]
    pub fn set_flcinprog(&mut self, val: u16) {
        let mask = 32u64 as u16;
        let val = val as u16 as u16;
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 5usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u16>(),
            );
        }
    }
    #[inline]
    pub fn reserved1(&self) -> u16 {
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        let mask = 192u64 as u16;
        let val = (unit_field_val & mask) >> 6usize;
        unsafe { ::core::mem::transmute(val as u16) }
    }
    #[inline]
    pub fn set_reserved1(&mut self, val: u16) {
        let mask = 192u64 as u16;
        let val = val as u16 as u16;
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 6usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u16>(),
            );
        }
    }
    #[inline]
    pub fn reserved2(&self) -> u16 {
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        let mask = 16128u64 as u16;
        let val = (unit_field_val & mask) >> 8usize;
        unsafe { ::core::mem::transmute(val as u16) }
    }
    #[inline]
    pub fn set_reserved2(&mut self, val: u16) {
        let mask = 16128u64 as u16;
        let val = val as u16 as u16;
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 8usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u16>(),
            );
        }
    }
    #[inline]
    pub fn fldesvalid(&self) -> u16 {
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        let mask = 16384u64 as u16;
        let val = (unit_field_val & mask) >> 14usize;
        unsafe { ::core::mem::transmute(val as u16) }
    }
    #[inline]
    pub fn set_fldesvalid(&mut self, val: u16) {
        let mask = 16384u64 as u16;
        let val = val as u16 as u16;
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 14usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u16>(),
            );
        }
    }
    #[inline]
    pub fn flockdn(&self) -> u16 {
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        let mask = 32768u64 as u16;
        let val = (unit_field_val & mask) >> 15usize;
        unsafe { ::core::mem::transmute(val as u16) }
    }
    #[inline]
    pub fn set_flockdn(&mut self, val: u16) {
        let mask = 32768u64 as u16;
        let val = val as u16 as u16;
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 15usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u16>(),
            );
        }
    }
    #[inline]
    pub fn new_bitfield_1(
        flcdone: u16,
        flcerr: u16,
        dael: u16,
        berasesz: u16,
        flcinprog: u16,
        reserved1: u16,
        reserved2: u16,
        fldesvalid: u16,
        flockdn: u16,
    ) -> u16 {
        ({
             ({
                  ({
                       ({
                            ({
                                 ({
                                      ({
                                           ({
                                                ({
                                                     0
                                                 } |
                                                     ((flcdone as u16 as u16) << 0usize) &
                                                         (1u64 as u16))
                                            } |
                                                ((flcerr as u16 as u16) << 1usize) & (2u64 as u16))
                                       } |
                                           ((dael as u16 as u16) << 2usize) & (4u64 as u16))
                                  } |
                                      ((berasesz as u16 as u16) << 3usize) & (24u64 as u16))
                             } |
                                 ((flcinprog as u16 as u16) << 5usize) & (32u64 as u16))
                        } |
                            ((reserved1 as u16 as u16) << 6usize) & (192u64 as u16))
                   } | ((reserved2 as u16 as u16) << 8usize) & (16128u64 as u16))
              } | ((fldesvalid as u16 as u16) << 14usize) & (16384u64 as u16))
         } | ((flockdn as u16 as u16) << 15usize) & (32768u64 as u16))
    }
}


#[repr(C)]
#[derive(Copy)]
pub union Ich8HwsFlashCtrl {
    pub hsf_ctrl: Ich8HsflCtrl,
    pub regval: u16,
    _bindgen_union_align: u16,
}
impl Clone for Ich8HwsFlashCtrl {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for Ich8HwsFlashCtrl {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
impl ::kernel::fmt::Debug for Ich8HwsFlashCtrl {
    fn fmt(&self, f: &mut ::kernel::fmt::Formatter) -> ::kernel::fmt::Result {
        write!(f, "Ich8HwsFlashCtrl {{ union }}")
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct Ich8HsflCtrl {
    pub _bitfield_1: [u8; 2usize],
    pub __bindgen_align: [u16; 0usize],
}
impl Clone for Ich8HsflCtrl {
    fn clone(&self) -> Self {
        *self
    }
}
impl Ich8HsflCtrl {
    #[inline]
    pub fn flcgo(&self) -> u16 {
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        let mask = 1u64 as u16;
        let val = (unit_field_val & mask) >> 0usize;
        unsafe { ::core::mem::transmute(val as u16) }
    }
    #[inline]
    pub fn set_flcgo(&mut self, val: u16) {
        let mask = 1u64 as u16;
        let val = val as u16 as u16;
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 0usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u16>(),
            );
        }
    }
    #[inline]
    pub fn flcycle(&self) -> u16 {
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        let mask = 6u64 as u16;
        let val = (unit_field_val & mask) >> 1usize;
        unsafe { ::core::mem::transmute(val as u16) }
    }
    #[inline]
    pub fn set_flcycle(&mut self, val: u16) {
        let mask = 6u64 as u16;
        let val = val as u16 as u16;
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 1usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u16>(),
            );
        }
    }
    #[inline]
    pub fn reserved(&self) -> u16 {
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        let mask = 248u64 as u16;
        let val = (unit_field_val & mask) >> 3usize;
        unsafe { ::core::mem::transmute(val as u16) }
    }
    #[inline]
    pub fn set_reserved(&mut self, val: u16) {
        let mask = 248u64 as u16;
        let val = val as u16 as u16;
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 3usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u16>(),
            );
        }
    }
    #[inline]
    pub fn fldbcount(&self) -> u16 {
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        let mask = 768u64 as u16;
        let val = (unit_field_val & mask) >> 8usize;
        unsafe { ::core::mem::transmute(val as u16) }
    }
    #[inline]
    pub fn set_fldbcount(&mut self, val: u16) {
        let mask = 768u64 as u16;
        let val = val as u16 as u16;
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 8usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u16>(),
            );
        }
    }
    #[inline]
    pub fn flockdn(&self) -> u16 {
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        let mask = 64512u64 as u16;
        let val = (unit_field_val & mask) >> 10usize;
        unsafe { ::core::mem::transmute(val as u16) }
    }
    #[inline]
    pub fn set_flockdn(&mut self, val: u16) {
        let mask = 64512u64 as u16;
        let val = val as u16 as u16;
        let mut unit_field_val: u16 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u16 as *mut u8,
                ::core::mem::size_of::<u16>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 10usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u16>(),
            );
        }
    }
    #[inline]
    pub fn new_bitfield_1(
        flcgo: u16,
        flcycle: u16,
        reserved: u16,
        fldbcount: u16,
        flockdn: u16,
    ) -> u16 {
        ({
             ({
                  ({
                       ({
                            ({
                                 0
                             } |
                                 ((flcgo as u16 as u16) << 0usize) & (1u64 as u16))
                        } |
                            ((flcycle as u16 as u16) << 1usize) & (6u64 as u16))
                   } | ((reserved as u16 as u16) << 3usize) & (248u64 as u16))
              } | ((fldbcount as u16 as u16) << 8usize) & (768u64 as u16))
         } | ((flockdn as u16 as u16) << 10usize) & (64512u64 as u16))
    }
}





#[repr(C, packed)]
#[derive(Debug, Default)]
pub struct ip {
    pub _bitfield_1: u8,
    pub ip_tos: u8,
    pub ip_len: u16,
    pub ip_id: u16,
    pub ip_off: u16,
    pub ip_ttl: u8,
    pub ip_p: u8,
    pub ip_sum: u16,
    pub ip_src: in_addr,
    pub ip_dst: in_addr,
}
impl ip {
    #[inline]
    pub fn ip_hl(&self) -> u8 {
        let mut unit_field_val: u8 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u8 as *mut u8,
                ::core::mem::size_of::<u8>(),
            )
        };
        let mask = 15u64 as u8;
        let val = (unit_field_val & mask) >> 0usize;
        unsafe { ::core::mem::transmute(val as u8) }
    }
    #[inline]
    pub fn set_ip_hl(&mut self, val: u8) {
        let mask = 15u64 as u8;
        let val = val as u8 as u8;
        let mut unit_field_val: u8 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u8 as *mut u8,
                ::core::mem::size_of::<u8>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 0usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u8>(),
            );
        }
    }
    #[inline]
    pub fn ip_v(&self) -> u8 {
        let mut unit_field_val: u8 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u8 as *mut u8,
                ::core::mem::size_of::<u8>(),
            )
        };
        let mask = 240u64 as u8;
        let val = (unit_field_val & mask) >> 4usize;
        unsafe { ::core::mem::transmute(val as u8) }
    }
    #[inline]
    pub fn set_ip_v(&mut self, val: u8) {
        let mask = 240u64 as u8;
        let val = val as u8 as u8;
        let mut unit_field_val: u8 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u8 as *mut u8,
                ::core::mem::size_of::<u8>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 4usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u8>(),
            );
        }
    }
    #[inline]
    pub fn new_bitfield_1(ip_hl: u8, ip_v: u8) -> u8 {
        ({
             ({
                  0
              } | ((ip_hl as u8 as u8) << 0usize) & (15u64 as u8))
         } | ((ip_v as u8 as u8) << 4usize) & (240u64 as u8))
    }
}



#[repr(C)]
#[derive(Debug, Default)]
pub struct udphdr {
    pub uh_sport: u16,
    pub uh_dport: u16,
    pub uh_ulen: u16,
    pub uh_sum: u16,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct tcphdr {
    pub th_sport: u16,
    pub th_dport: u16,
    pub th_seq: u32, // tcp_seq = u32
    pub th_ack: u32,
    pub _bitfield_1: u8,
    pub th_flags: u8,
    pub th_win: u16,
    pub th_sum: u16,
    pub th_urp: u16,
}
impl tcphdr {
    #[inline]
    pub fn th_x2(&self) -> u8 {
        let mut unit_field_val: u8 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u8 as *mut u8,
                ::core::mem::size_of::<u8>(),
            )
        };
        let mask = 15u64 as u8;
        let val = (unit_field_val & mask) >> 0usize;
        unsafe { ::core::mem::transmute(val as u8) }
    }
    #[inline]
    pub fn set_th_x2(&mut self, val: u8) {
        let mask = 15u64 as u8;
        let val = val as u8 as u8;
        let mut unit_field_val: u8 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u8 as *mut u8,
                ::core::mem::size_of::<u8>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 0usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u8>(),
            );
        }
    }
    #[inline]
    pub fn th_off(&self) -> u8 {
        let mut unit_field_val: u8 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u8 as *mut u8,
                ::core::mem::size_of::<u8>(),
            )
        };
        let mask = 240u64 as u8;
        let val = (unit_field_val & mask) >> 4usize;
        unsafe { ::core::mem::transmute(val as u8) }
    }
    #[inline]
    pub fn set_th_off(&mut self, val: u8) {
        let mask = 240u64 as u8;
        let val = val as u8 as u8;
        let mut unit_field_val: u8 = unsafe { ::core::mem::uninitialized() };
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &self._bitfield_1 as *const _ as *const u8,
                &mut unit_field_val as *mut u8 as *mut u8,
                ::core::mem::size_of::<u8>(),
            )
        };
        unit_field_val &= !mask;
        unit_field_val |= (val << 4usize) & mask;
        unsafe {
            ::core::ptr::copy_nonoverlapping(
                &unit_field_val as *const _ as *const u8,
                &mut self._bitfield_1 as *mut _ as *mut u8,
                ::core::mem::size_of::<u8>(),
            );
        }
    }
    #[inline]
    pub fn new_bitfield_1(th_x2: u8, th_off: u8) -> u8 {
        ({
             ({
                  0
              } | ((th_x2 as u8 as u8) << 0usize) & (15u64 as u8))
         } | ((th_off as u8 as u8) << 4usize) & (240u64 as u8))
    }
}




#[repr(C)]
#[derive(Debug, Default)]
pub struct in_addr {
    pub s_addr: u32,
}







#[derive(Copy)]
pub struct e1000_tx_desc {
    pub buffer_addr: u64,
    pub lower: e1000_tx_desc_u1,
    pub upper: e1000_tx_desc_u2,
}
impl Clone for e1000_tx_desc {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for e1000_tx_desc {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
impl ::kernel::fmt::Debug for e1000_tx_desc {
    fn fmt(&self, f: &mut ::kernel::fmt::Formatter) -> ::kernel::fmt::Result {
        write!(
            f,
            "e1000_tx_desc {{ buffer_addr: {:?}, lower: {:?}, upper: {:?} }}",
            self.buffer_addr,
            self.lower,
            self.upper
        )
    }
}

#[repr(C)]
#[derive(Copy)]
pub union e1000_tx_desc_u1 {
    pub data: u32,
    pub flags: e1000_tx_desc_u1_u1,
    _bindgen_union_align: u32,
}
impl Clone for e1000_tx_desc_u1 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for e1000_tx_desc_u1 {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
impl ::kernel::fmt::Debug for e1000_tx_desc_u1 {
    fn fmt(&self, f: &mut ::kernel::fmt::Formatter) -> ::kernel::fmt::Result {
        write!(f, "e1000_tx_desc_u1 {{ union }}")
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct e1000_tx_desc_u1_u1 {
    pub length: u16,
    pub cso: u8,
    pub cmd: u8,
}
impl Clone for e1000_tx_desc_u1_u1 {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
#[derive(Copy)]
pub union e1000_tx_desc_u2 {
    pub data: u32,
    pub fields: e1000_tx_desc_u2_u1,
    _bindgen_union_align: u32,
}
impl Clone for e1000_tx_desc_u2 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for e1000_tx_desc_u2 {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
impl ::kernel::fmt::Debug for e1000_tx_desc_u2 {
    fn fmt(&self, f: &mut ::kernel::fmt::Formatter) -> ::kernel::fmt::Result {
        write!(f, "e1000_tx_desc_u2 {{ union }}")
    }
}


#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct e1000_tx_desc_u2_u1 {
    pub status: u8,
    pub css: u8,
    pub special: u16,
}
impl Clone for e1000_tx_desc_u2_u1 {
    fn clone(&self) -> Self {
        *self
    }
}








#[repr(C)]
#[derive(Copy)]
pub union e1000_rx_desc_extended {
    pub read: e1000_rx_desc_extended_u1,
    pub wb: e1000_rx_desc_extended_u2,
    _bindgen_union_align: [u64; 2usize],
}
#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct e1000_rx_desc_extended_u1 {
    pub buffer_addr: u64,
    pub reserved: u64,
}
impl Clone for e1000_rx_desc_extended_u1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[derive(Copy)]
pub struct e1000_rx_desc_extended_u2 {
    pub lower: e1000_rx_desc_extended_u2_u1,
    pub upper: e1000_rx_desc_extended_u2_u2,
}
#[repr(C)]
#[derive(Copy)]
pub struct e1000_rx_desc_extended_u2_u1 {
    pub mrq: u32,
    pub hi_dword: e1000_rx_desc_extended_u2_u1_u1,
}
#[repr(C)]
#[derive(Copy)]
pub union e1000_rx_desc_extended_u2_u1_u1 {
    pub rss: u32,
    pub csum_ip: e1000_rx_desc_extended_u2_u1_u1_u1,
    _bindgen_union_align: u32,
}
#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct e1000_rx_desc_extended_u2_u1_u1_u1 {
    pub ip_id: u16,
    pub csum: u16,
}
impl Clone for e1000_rx_desc_extended_u2_u1_u1_u1 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Clone for e1000_rx_desc_extended_u2_u1_u1 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for e1000_rx_desc_extended_u2_u1_u1 {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
impl ::kernel::fmt::Debug for e1000_rx_desc_extended_u2_u1_u1 {
    fn fmt(&self, f: &mut ::kernel::fmt::Formatter) -> ::kernel::fmt::Result {
        write!(f, "e1000_rx_desc_extended_u2_u1_u1 {{ union }}")
    }
}
impl Clone for e1000_rx_desc_extended_u2_u1 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for e1000_rx_desc_extended_u2_u1 {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
impl ::kernel::fmt::Debug for e1000_rx_desc_extended_u2_u1 {
    fn fmt(&self, f: &mut ::kernel::fmt::Formatter) -> ::kernel::fmt::Result {
        write!(
            f,
            "e1000_rx_desc_extended_u2_u1 {{ mrq: {:?}, hi_dword: {:?} }}",
            self.mrq,
            self.hi_dword
        )
    }
}
#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct e1000_rx_desc_extended_u2_u2 {
    pub status_error: u32,
    pub length: u16,
    pub vlan: u16,
}
impl Clone for e1000_rx_desc_extended_u2_u2 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Clone for e1000_rx_desc_extended_u2 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for e1000_rx_desc_extended_u2 {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
impl ::kernel::fmt::Debug for e1000_rx_desc_extended_u2 {
    fn fmt(&self, f: &mut ::kernel::fmt::Formatter) -> ::kernel::fmt::Result {
        write!(
            f,
            "e1000_rx_desc_extended_u2 {{ lower: {:?}, upper: {:?} }}",
            self.lower,
            self.upper
        )
    }
}
impl Clone for e1000_rx_desc_extended {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for e1000_rx_desc_extended {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
impl ::kernel::fmt::Debug for e1000_rx_desc_extended {
    fn fmt(&self, f: &mut ::kernel::fmt::Formatter) -> ::kernel::fmt::Result {
        write!(f, "e1000_rx_desc_extended {{ union }}")
    }
}





#[repr(C)]
#[derive(Copy)]
pub struct e1000_context_desc {
    pub lower_setup: e1000_context_desc__bindgen_ty_1,
    pub upper_setup: e1000_context_desc__bindgen_ty_2,
    pub cmd_and_length: u32,
    pub tcp_seg_setup: e1000_context_desc__bindgen_ty_3,
}
#[repr(C)]
#[derive(Copy)]
pub union e1000_context_desc__bindgen_ty_1 {
    pub ip_config: u32,
    pub ip_fields: e1000_context_desc__bindgen_ty_1__bindgen_ty_1,
    _bindgen_union_align: u32,
}
#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct e1000_context_desc__bindgen_ty_1__bindgen_ty_1 {
    pub ipcss: u8,
    pub ipcso: u8,
    pub ipcse: u16,
}
impl Clone for e1000_context_desc__bindgen_ty_1__bindgen_ty_1 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Clone for e1000_context_desc__bindgen_ty_1 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for e1000_context_desc__bindgen_ty_1 {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
impl ::kernel::fmt::Debug for e1000_context_desc__bindgen_ty_1 {
    fn fmt(&self, f: &mut ::kernel::fmt::Formatter) -> ::kernel::fmt::Result {
        write!(f, "e1000_context_desc__bindgen_ty_1 {{ union }}")
    }
}
#[repr(C)]
#[derive(Copy)]
pub union e1000_context_desc__bindgen_ty_2 {
    pub tcp_config: u32,
    pub tcp_fields: e1000_context_desc__bindgen_ty_2__bindgen_ty_1,
    _bindgen_union_align: u32,
}
#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct e1000_context_desc__bindgen_ty_2__bindgen_ty_1 {
    pub tucss: u8,
    pub tucso: u8,
    pub tucse: u16,
}
impl Clone for e1000_context_desc__bindgen_ty_2__bindgen_ty_1 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Clone for e1000_context_desc__bindgen_ty_2 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for e1000_context_desc__bindgen_ty_2 {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
impl ::kernel::fmt::Debug for e1000_context_desc__bindgen_ty_2 {
    fn fmt(&self, f: &mut ::kernel::fmt::Formatter) -> ::kernel::fmt::Result {
        write!(f, "e1000_context_desc__bindgen_ty_2 {{ union }}")
    }
}
#[repr(C)]
#[derive(Copy)]
pub union e1000_context_desc__bindgen_ty_3 {
    pub data: u32,
    pub fields: e1000_context_desc__bindgen_ty_3__bindgen_ty_1,
    _bindgen_union_align: u32,
}
#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct e1000_context_desc__bindgen_ty_3__bindgen_ty_1 {
    pub status: u8,
    pub hdr_len: u8,
    pub mss: u16,
}
impl Clone for e1000_context_desc__bindgen_ty_3__bindgen_ty_1 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Clone for e1000_context_desc__bindgen_ty_3 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for e1000_context_desc__bindgen_ty_3 {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
impl ::kernel::fmt::Debug for e1000_context_desc__bindgen_ty_3 {
    fn fmt(&self, f: &mut ::kernel::fmt::Formatter) -> ::kernel::fmt::Result {
        write!(f, "e1000_context_desc__bindgen_ty_3 {{ union }}")
    }
}
impl Clone for e1000_context_desc {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for e1000_context_desc {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
impl ::kernel::fmt::Debug for e1000_context_desc {
    fn fmt(&self, f: &mut ::kernel::fmt::Formatter) -> ::kernel::fmt::Result {
        write!(
            f,
            "e1000_context_desc {{ lower_setup: {:?}, upper_setup: {:?}, cmd_and_length: {:?}, tcp_seg_setup: {:?} }}",
            self.lower_setup,
            self.upper_setup,
            self.cmd_and_length,
            self.tcp_seg_setup
        )
    }
}




// TODO: Move to kernel?
#[repr(C)]
#[derive(Debug, Default)]
pub struct Witness {
    pub address: u8,
}

// TODO: Move to kernel?
#[repr(C)]
#[derive(Debug)]
pub struct LockObject {
    pub lo_name: String,
    pub lo_flags: u32,
    pub lo_data: u32,
    pub lo_witness: Witness,
}

// TODO: Move to kernel?
#[repr(C)]
#[derive(Debug)]
pub struct Mtx {
    pub lock_object: LockObject,
    pub mtx_lock: usize,
}

/*
/* Receive Descriptor */
struct e1000_rx_desc {
	__le64 buffer_addr; /* Address of the descriptor's data buffer */
	__le16 length;      /* Length of data DMAed into data buffer */
	__le16 csum; /* Packet checksum */
	u8  status;  /* Descriptor status */
	u8  errors;  /* Descriptor Errors */
	__le16 special;
};
 */

#[repr(C)]
#[derive(Debug, Default, Copy)]
pub struct e1000_rx_desc {
    pub buffer_addr: u64,
    pub length: u16,
    pub csum: u16,
    pub status: u8,
    pub errors: u8,
    pub special: u16,
}
impl Clone for e1000_rx_desc {
    fn clone(&self) -> Self {
        *self
    }
}


#[derive(Debug)]
pub struct TxQueue {
    // pub adapter: *mut adapter,
    pub msix: u32,
    pub eims: u32,
    pub me: u32,
    pub txr: TxRing,
}
impl Default for TxQueue {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
// impl Clone for TxQueue {
//     fn clone(&self) -> Self {
//         *self
//     }
// }
#[derive(Debug)]
pub struct RxQueue {
    // pub adapter: *mut adapter,
    pub me: u32,
    pub msix: u32,
    pub eims: u32,
    pub rxr: RxRing,
    pub irqs: u64,
    pub que_irq: IfIrq,
}
impl Default for RxQueue {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}


#[derive(Debug)]
pub struct TxRing {
    // pub adapter: *mut adapter,
    pub tx_base: *mut e1000_tx_desc,
    pub tx_paddr: u64,
    pub tx_rsq: Box<[u16]>,
    pub tx_tso: bool,
    pub me: u8,
    pub tx_rs_cidx: u16,
    pub tx_rs_pidx: u16,
    pub tx_cidx_processed: u16,
    pub tag: *mut ::kernel::sys::raw::c_void,
    pub res: Resource,
    pub tx_irq: u64,
    pub csum_flags: i32,
    pub csum_lhlen: i32,
    pub csum_iphlen: i32,
    pub csum_thlen: i32,
    pub csum_mss: i32,
    pub csum_pktlen: i32,
    pub csum_txd_upper: u32,
    pub csum_txd_lower: u32,
}
impl TxRing {
    pub fn clear_checksum(&mut self) {
        self.csum_flags = 0;
        self.csum_iphlen = 0;
        self.csum_lhlen = 0;
        self.csum_mss = 0;
        self.csum_pktlen = 0;
        self.csum_txd_lower = 0;
        self.csum_txd_upper = 0;
    }
    pub fn txd_tx_desc_slice(&mut self, len: usize) -> &mut [e1000_tx_desc] {
        unsafe { kernel::slice::from_raw_parts_mut(self.tx_base, len) }
    }
    pub fn txd_context_desc_slice(&mut self, len: usize) -> &mut [e1000_context_desc] {
        let cast_ptr = self.tx_base as *mut e1000_context_desc;
        unsafe { kernel::slice::from_raw_parts_mut(cast_ptr, len) }
    }
}
impl Default for TxRing {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
// impl Clone for TxRing {
//     fn clone(&self) -> Self {
//         *self
//     }
// }


#[derive(Debug)]
pub struct RxRing {
    // pub adapter: *mut adapter,
    // pub que: *mut RxQueue,
    pub me: u32,
    pub payload: u32,
    pub rx_base: *mut e1000_rx_desc_extended,
    pub rx_paddr: u64,
    pub tag: *mut ::kernel::sys::raw::c_void,
    pub res: Resource,
    pub discard: bool,
    pub rx_irq: ::kernel::sys::raw::c_ulong,
    pub rx_discarded: ::kernel::sys::raw::c_ulong,
    pub rx_packets: ::kernel::sys::raw::c_ulong,
    pub rx_bytes: ::kernel::sys::raw::c_ulong,
}
impl RxRing {
    pub fn rxd_rx_desc_slice(&mut self, len: usize) -> &mut [e1000_rx_desc] {
        let rx_base: *mut e1000_rx_desc = self.rx_base as *mut e1000_rx_desc;
        unsafe { kernel::slice::from_raw_parts_mut(rx_base, len) }
    }
    pub fn rxd_rx_desc_extended_slice(&mut self, len: usize) -> &mut [e1000_rx_desc_extended] {
        unsafe { kernel::slice::from_raw_parts_mut(self.rx_base, len) }
    }
}
impl Default for RxRing {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
// impl Clone for RxRing {
//     fn clone(&self) -> Self {
//         *self
//     }
// }


#[repr(C)]
#[derive(Debug, Copy)]
pub struct if_txrx {
    pub ift_txd_encap:
        ::core::option::Option<
            unsafe extern "C" fn(arg1: *mut ::kernel::sys::raw::c_void,
                                 arg2: if_pkt_info_t)
                                 -> ::kernel::sys::raw::c_int,
        >,
    pub ift_txd_flush:
        ::core::option::Option<
            unsafe extern "C" fn(arg1: *mut ::kernel::sys::raw::c_void,
                                 arg2: u16,
                                 pidx: qidx_t),
        >,
    pub ift_txd_credits_update:
        ::core::option::Option<
            unsafe extern "C" fn(arg1: *mut ::kernel::sys::raw::c_void,
                                 qsidx: u16,
                                 clear: bool_)
                                 -> ::kernel::sys::raw::c_int,
        >,
    pub ift_rxd_available:
        ::core::option::Option<
            unsafe extern "C" fn(arg1: *mut ::kernel::sys::raw::c_void,
                                 qsidx: u16,
                                 pidx: qidx_t,
                                 budget: qidx_t)
                                 -> ::kernel::sys::raw::c_int,
        >,
    pub ift_rxd_pkt_get:
        ::core::option::Option<
            unsafe extern "C" fn(arg1: *mut ::kernel::sys::raw::c_void,
                                 ri: if_rxd_info_t)
                                 -> ::kernel::sys::raw::c_int,
        >,
    pub ift_rxd_refill:
        ::core::option::Option<
            unsafe extern "C" fn(arg1: *mut ::kernel::sys::raw::c_void,
                                 iru: *mut if_rxd_update),
        >,
    pub ift_rxd_flush:
        ::core::option::Option<
            unsafe extern "C" fn(arg1: *mut ::kernel::sys::raw::c_void,
                                 qsidx: u16,
                                 flidx: u8,
                                 pidx: qidx_t),
        >,
    pub ift_legacy_intr:
        ::core::option::Option<
            unsafe extern "C" fn(arg1: *mut ::kernel::sys::raw::c_void)
                                 -> ::kernel::sys::raw::c_int,
        >,
}
impl Clone for if_txrx {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for if_txrx {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
pub type if_txrx_t = *mut if_txrx;



#[derive(Debug, Default)]
pub struct HwStats {
    pub crcerrs: u64,
    pub algnerrc: u64,
    pub symerrs: u64,
    pub rxerrc: u64,
    pub mpc: u64,
    pub scc: u64,
    pub ecol: u64,
    pub mcc: u64,
    pub latecol: u64,
    pub colc: u64,
    pub dc: u64,
    pub tncrs: u64,
    pub sec: u64,
    pub cexterr: u64,
    pub rlec: u64,
    pub xonrxc: u64,
    pub xontxc: u64,
    pub xoffrxc: u64,
    pub xofftxc: u64,
    pub fcruc: u64,
    pub prc64: u64,
    pub prc127: u64,
    pub prc255: u64,
    pub prc511: u64,
    pub prc1023: u64,
    pub prc1522: u64,
    pub gprc: u64,
    pub bprc: u64,
    pub mprc: u64,
    pub gptc: u64,
    pub gorc: u64,
    pub gotc: u64,
    pub rnbc: u64,
    pub ruc: u64,
    pub rfc: u64,
    pub roc: u64,
    pub rjc: u64,
    pub mgprc: u64,
    pub mgpdc: u64,
    pub mgptc: u64,
    pub tor: u64,
    pub tot: u64,
    pub tpr: u64,
    pub tpt: u64,
    pub ptc64: u64,
    pub ptc127: u64,
    pub ptc255: u64,
    pub ptc511: u64,
    pub ptc1023: u64,
    pub ptc1522: u64,
    pub mptc: u64,
    pub bptc: u64,
    pub tsctc: u64,
    pub tsctfc: u64,
    pub iac: u64,
    pub icrxptc: u64,
    pub icrxatc: u64,
    pub ictxptc: u64,
    pub ictxatc: u64,
    pub ictxqec: u64,
    pub ictxqmtc: u64,
    pub icrxdmtc: u64,
    pub icrxoc: u64,
    pub cbtmpc: u64,
    pub htdpmc: u64,
    pub cbrdpc: u64,
    pub cbrmpc: u64,
    pub rpthc: u64,
    pub hgptc: u64,
    pub htcbdpc: u64,
    pub hgorc: u64,
    pub hgotc: u64,
    pub lenerrs: u64,
    pub scvpc: u64,
    pub hrmpc: u64,
    pub doosync: u64,
    pub o2bgptc: u64,
    pub o2bspc: u64,
    pub b2ospc: u64,
    pub b2ogprc: u64,
}

#[derive(Debug)]
pub struct IntDelayInfo {
    // pub adapter: *mut adapter,
    pub offset: i32,
    pub value: u32,
}




#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MediaType {
    Unknown,
    Copper,
    Fiber,
    InternalSerdes,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum MacType {
    Mac_undefined = 0,
    Mac_82542 = 1,
    Mac_82543 = 2,
    Mac_82544 = 3,
    Mac_82540 = 4,
    Mac_82545 = 5,
    Mac_82545_rev_3 = 6,
    Mac_82546 = 7,
    Mac_82546_rev_3 = 8,
    Mac_82541 = 9,
    Mac_82541_rev_2 = 10,
    Mac_82547 = 11, // EM_MAC_MIN
    Mac_82547_rev_2 = 12,
    Mac_82571 = 13,
    Mac_82572 = 14,
    Mac_82573 = 15,
    Mac_82574 = 16,
    Mac_82583 = 17,
    Mac_80003es2lan = 18,
    Mac_ich8lan = 19,
    Mac_ich9lan = 20,
    Mac_ich10lan = 21,
    Mac_pchlan = 22,
    Mac_pch2lan = 23,
    Mac_pch_lpt = 24,
    Mac_pch_spt = 25,
    Mac_pch_cnp = 26,
    Mac_82575 = 27, // IGB_MAC_MIN
    Mac_82576 = 28,
    Mac_82580 = 29,
    Mac_i350 = 30,
    Mac_i354 = 31,
    Mac_i210 = 32,
    Mac_i211 = 33,
    Mac_vfadapt = 34,
    Mac_vfadapt_i350 = 35,
}
impl MacType {
    pub const FIRST: MacType = MacType::Mac_82542;
    pub const LAST: MacType = MacType::Mac_vfadapt_i350;
    pub const EM_MAC_MIN: MacType = MacType::Mac_82547;
    pub const IGB_MAC_MIN: MacType = MacType::Mac_82575;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BusType {
    Unknown,
    Pci,
    Pcix,
    Pci_express,
    Reserved,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BusSpeed {
    Unknown,
    Speed_33,
    Speed_66,
    Speed_100,
    Speed_120,
    Speed_133,
    Speed_2500,
    Speed_5000,
    Reserved,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BusWidth {
    // e1000_bus_width_unknown = 0,
    // e1000_bus_width_pcie_x1 = 1,
    // e1000_bus_width_pcie_x2 = 2,
    // e1000_bus_width_pcie_x4 = 4,
    // e1000_bus_width_pcie_x8 = 8,
    // e1000_bus_width_32 = 9,
    // e1000_bus_width_64 = 10,
    // e1000_bus_width_reserved = 11,
    Unknown = 0,
    Width_pcie_x1 = 1,
    Width_pcie_x2 = 2,
    Width_pcie_x4 = 4,
    Width_pcie_x8 = 8,
    Width_32 = 9,
    Width_64 = 10,
    Reserved = 11,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SerdesLinkState {
    Down = 0,
    AutonegProgress = 1,
    AutonegComplete = 2,
    ForcedUp = 3,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SmartSpeed {
    Default = 0,
    On = 1,
    Off = 2,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RevPolarity {
    Normal = 0,
    Reversed = 1,
    Undefined = 255,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MsType {
    HwDefault = 0,
    ForceMaster = 1,
    ForceSlave = 2,
    Auto = 3,
}
impl MsType {
    /* PHY master/slave setting */
    pub const EM_MASTER_SLAVE: MsType = MsType::HwDefault;
}


#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GbRxStatus {
    NotOk = 0,
    Ok = 1,
    Undefined = 255,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum PhyType {
    Type_unknown = 0,
    Type_none = 1,
    Type_m88 = 2,
    Type_igp = 3,
    Type_igp_2 = 4,
    Type_gg82563 = 5,
    Type_igp_3 = 6,
    Type_ife = 7,
    Type_bm = 8,
    Type_82578 = 9,
    Type_82577 = 10,
    Type_82579 = 11,
    Type_i217 = 12,
    Type_82580 = 13,
    Type_vf = 14,
    Type_i210 = 15,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum NvmType {
    Unknown = 0,
    None = 1,
    EepromSpi = 2,
    EepromMicrowire = 3,
    FlashHw = 4,
    Invm = 5,
    FlashSw = 6,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum NvmOverride {
    None = 0,
    SpiSmall = 1,
    SpiLarge = 2,
    MicrowireSmall = 3,
    MicrowireLarge = 4,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MngMode {
    None = 0,
    Asf = 1,
    Pt = 2,
    Ipmi = 3,
    HostIfOnly = 4,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct MbxStats {
    pub msgs_tx: u32,
    pub msgs_rx: u32,
    pub acks: u32,
    pub reqs: u32,
    pub rsts: u32,
}
