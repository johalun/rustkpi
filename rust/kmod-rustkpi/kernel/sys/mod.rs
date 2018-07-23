
pub mod raw;

pub mod bus_sys;
pub mod callout_sys;
pub mod conf_sys;
pub mod iflib_sys;
pub mod kernel_sys;
pub mod kobj_sys;
pub mod kthread_sys;
pub mod malloc_sys;
pub mod mbuf_sys;
pub mod module_sys;
pub mod mutex_sys;
pub mod param_sys;
pub mod pci_sys;
pub mod sysctl_sys;
pub mod systm_sys;
pub mod types_sys;
pub mod uio_sys;
pub mod unistd_sys;

pub use self::raw::*;


pub use self::param_sys::__FreeBSD_version;

pub use self::kernel_sys::sysinit;
pub use self::kernel_sys::sysinit_sub_id;
pub use self::kernel_sys::sysinit_elem_order;

pub use self::module_sys::mod_metadata;
pub use self::module_sys::mod_depend;
pub use self::module_sys::mod_version;
pub use self::module_sys::module_register_init;
pub use self::module_sys::moduledata_t;
pub use self::module_sys::module_t;
pub use self::module_sys::MDT_STRUCT_VERSION;
pub use self::module_sys::MDT_DEPEND;
pub use self::module_sys::MDT_VERSION;
pub use self::module_sys::MDT_MODULE;

pub use self::malloc_sys::malloc_type;
pub use self::malloc_sys::M_MAGIC;

pub use self::sysctl_sys::CTLTYPE_NODE;
pub use self::sysctl_sys::CTLTYPE_INT;
pub use self::sysctl_sys::OID_AUTO;
pub use self::sysctl_sys::sysctl_oid;

pub use self::systm_sys::uprintf;
pub use self::systm_sys::printf;


unsafe impl Sync for self::moduledata_t {}
unsafe impl Sync for self::mod_metadata {}
unsafe impl Sync for self::mod_version {}
unsafe impl Sync for self::sysinit {}
unsafe impl Sync for self::sysctl_oid {}

unsafe impl Sync for self::iflib_sys::devclass {}
unsafe impl Sync for self::iflib_sys::driver_module_data {}
unsafe impl Sync for self::iflib_sys::kobj_ops {}
unsafe impl Sync for self::iflib_sys::kobj_class {}
unsafe impl Sync for self::iflib_sys::kobj_method {}
unsafe impl Sync for self::iflib_sys::kobjop_desc {}



#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ModEventType {
    Load = 0,
    Unload = 1,
    Shutdown = 2,
    Quiesce = 3,
    Unknown = 4,
}
impl From<i32> for ModEventType {
    fn from(i: i32) -> Self {
        match i {
            0 => ModEventType::Load,
            1 => ModEventType::Unload,
            2 => ModEventType::Shutdown,
            3 => ModEventType::Quiesce,
            _ => ModEventType::Unknown,
        }
    }
}
