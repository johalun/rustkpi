
use sys::e1000::*;


pub const fn em_bar_type(v: u32) -> u32 {
    v & EM_BAR_TYPE_MASK
}

pub const fn em_bar_mem_type(v: u32) -> u32 {
    v & EM_BAR_MEM_TYPE_MASK
}
