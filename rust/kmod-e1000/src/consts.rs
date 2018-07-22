
use sys::e1000::*;


// #define EM_BAR_TYPE(v)		((v) & EM_BAR_TYPE_MASK)
// #define EM_BAR_MEM_TYPE(v)	((v) & EM_BAR_MEM_TYPE_MASK)

pub const fn em_bar_type(v: u32) -> u32 {
    v & EM_BAR_TYPE_MASK
}

pub const fn em_bar_mem_type(v: u32) -> u32 {
    v & EM_BAR_MEM_TYPE_MASK
}
