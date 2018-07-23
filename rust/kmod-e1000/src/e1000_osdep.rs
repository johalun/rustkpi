
use kernel;
use kernel::ptr::NonNull;
use kernel::sys::raw::*;
use kernel::prelude::v1::*;

use kernel::sys::iflib_sys::device;
use kernel::sys::iflib_sys::device_get_parent;

use kernel::sys::bus_sys::bus_alloc_resource;
use kernel::sys::bus_sys::bus_release_resource;
use kernel::sys::bus_sys::resource;
use kernel::sys::pci_sys::PCIY_EXPRESS;

pub use kernel::sys::bus_sys::SYS_RES_MEMORY;
pub use kernel::sys::bus_sys::SYS_RES_IOPORT;
pub use kernel::sys::bus_sys::RF_ACTIVE;
pub use kernel::sys::bus_sys::bus_size_t;
pub use kernel::sys::bus_sys::bus_space_tag_t;
pub use kernel::sys::bus_sys::bus_space_handle_t;
pub use kernel::sys::bus_sys::rman_get_bustag;
pub use kernel::sys::bus_sys::rman_get_bushandle;
pub use kernel::sys::bus_sys::rman_get_size;


use sys::e1000::*;
use iflib::*;
use hw::*;
use consts::*;
use bridge::*;
use adapter::*;
use e1000_82542;

extern "C" {
    pub fn rust_pci_find_cap(dev: *mut device, reg: c_int, out: *mut c_int) -> i32;
    pub fn rust_pci_read_config(
        dev: *mut device,
        child: *mut device,
        reg: c_int,
        width: c_int,
    ) -> u32;
    pub fn rust_pci_get_vendor(dev: *mut device) -> u32;
    pub fn rust_pci_get_device(dev: *mut device) -> u32;
    pub fn rust_pci_get_subvendor(dev: *mut device) -> u32;
    pub fn rust_pci_get_subdevice(dev: *mut device) -> u32;

    pub fn rust_bus_space_read_2(
        tag: bus_space_tag_t,
        handle: bus_space_handle_t,
        offset: bus_size_t,
    ) -> u16;

    pub fn rust_bus_space_read_4(
        tag: bus_space_tag_t,
        handle: bus_space_handle_t,
        offset: bus_size_t,
    ) -> u32;


    pub fn rust_bus_space_write_2(
        tag: bus_space_tag_t,
        handle: bus_space_handle_t,
        offset: bus_size_t,
        value: u16,
    );

    pub fn rust_bus_space_write_4(
        tag: bus_space_tag_t,
        handle: bus_space_handle_t,
        offset: bus_size_t,
        value: u32,
    );

    pub fn rust_usec_delay(usecs: usize);

}


pub fn register(hw: &Hardware, reg: u32) -> u32 {
    // e1000_println!();
    match hw.mac.mac_type {
        x if x >= MacType::Mac_82543 => reg,
        _ => e1000_82542::translate_register(reg),
    }
}

pub fn do_write_flush(adapter: &Adapter) {
    do_read_register(adapter, E1000_STATUS);
}

// TODO: replace bus_* ffi with pure Rust functions?

pub fn do_read_register(adapter: &Adapter, reg: u32) -> u32 {
    // e1000_println!();
    unsafe {
        rust_bus_space_read_4(
            adapter.osdep.mem_bus_space_tag,
            adapter.osdep.mem_bus_space_handle,
            reg as bus_size_t,
        )
    }
}

pub fn do_write_register(adapter: &Adapter, reg: u32, value: u32) {
    // e1000_println!();
    unsafe {
        rust_bus_space_write_4(
            adapter.osdep.mem_bus_space_tag,
            adapter.osdep.mem_bus_space_handle,
            reg as bus_size_t,
            value,
        );
    }
}

pub fn do_write_register_array(adapter: &Adapter, reg: u32, index: u32, value: u32) {
    // e1000_println!();
    unsafe {
        rust_bus_space_write_4(
            adapter.osdep.mem_bus_space_tag,
            adapter.osdep.mem_bus_space_handle,
            (reg + (index << 2)) as bus_size_t,
            value,
        );
    }
}




pub fn do_write_register_io(adapter: &Adapter, reg: u32, value: u32) {
    // e1000_println!();
    unsafe {
        rust_bus_space_write_4(
            adapter.osdep.io_bus_space_tag,
            adapter.osdep.io_bus_space_handle,
            adapter.hw.io_base,
            reg,
        );
        rust_bus_space_write_4(
            adapter.osdep.io_bus_space_tag,
            adapter.osdep.io_bus_space_handle,
            adapter.hw.io_base + 4,
            value,
        );
    }
}


pub fn do_usec_delay(usecs: usize) {
    // e1000_println!();
    unsafe {
        rust_usec_delay(usecs);
    }
}

pub fn do_msec_delay(msecs: usize) {
    // e1000_println!();
    unsafe {
        rust_usec_delay(1000 * msecs);
    }
}




#[derive(Debug)]
pub struct Resource {
    pub inner: NonNull<resource>,
    dev: NonNull<device>,
    systype: i32,
    rid: i32,
}
impl Resource {
    pub fn rman_get_bustag(&self) -> bus_space_tag_t {
        e1000_println!();
        let res: *mut resource = self.inner.as_ptr();
        unsafe { rman_get_bustag(res) }
    }
    pub fn rman_get_bushandle(&self) -> bus_space_handle_t {
        e1000_println!();
        let res: *mut resource = self.inner.as_ptr();
        unsafe { rman_get_bushandle(res) }
    }
    pub fn rman_get_size(&self) -> u64 {
        e1000_println!();
        let res: *mut resource = self.inner.as_ptr();
        unsafe { rman_get_size(res) }
    }
}
impl kernel::ops::Deref for Resource {
    type Target = resource;
    fn deref(&self) -> &resource {
        unsafe { self.inner.as_ref() }
    }
}
impl kernel::ops::DerefMut for Resource {
    fn deref_mut(&mut self) -> &mut resource {
        unsafe { self.inner.as_mut() }
    }
}
impl Drop for Resource {
    fn drop(&mut self) {
        unsafe {
            bus_release_resource(
                self.dev.as_ptr() as *mut kernel::sys::bus_sys::device,
                self.systype,
                self.rid,
                self.inner.as_ptr(),
            );
        }
    }
}

#[derive(Debug)]
pub struct OsDep {
    pub mem_bus_space_tag: bus_space_tag_t,
    pub mem_bus_space_handle: bus_space_handle_t,
    pub io_bus_space_tag: bus_space_tag_t,
    pub io_bus_space_handle: bus_space_handle_t,
    pub flash_bus_space_tag: bus_space_tag_t,
    pub flash_bus_space_handle: bus_space_handle_t,
}
impl Drop for OsDep {
    fn drop(&mut self) {
        // Clean up?
    }
}


#[derive(Debug)]
pub struct PciDevice {
    pub inner: NonNull<device>, // device internal struct
}
impl PciDevice {
    pub fn pci_read_config(&self, reg: u32, width: u32) -> u32 {
        e1000_println!();
        let child: *mut device = self.inner.as_ptr();
        let parent: *mut device = unsafe { device_get_parent(child) };
        let ret = unsafe { rust_pci_read_config(parent, child, reg as c_int, width as c_int) };
        ret
    }
    pub fn read_pcie_cap_reg(&mut self, reg: u32, value: &mut u16) -> AdResult {
        e1000_println!();
        let child: *mut device = self.inner.as_ptr();
        let parent: *mut device = unsafe { device_get_parent(child) };
        let mut offset = 0;
        unsafe { rust_pci_find_cap(child, PCIY_EXPRESS as c_int, &mut offset) };
        *value = unsafe { rust_pci_read_config(parent, child, offset + reg as c_int, 2) as u16 };
        Ok(())
    }

    pub fn pci_get_vendor(&self) -> u32 {
        e1000_println!();
        let dev: *mut device = self.inner.as_ptr();
        let ret = unsafe { rust_pci_get_vendor(dev) };
        ret
    }
    pub fn pci_get_device(&self) -> u32 {
        e1000_println!();
        let dev: *mut device = self.inner.as_ptr();
        let ret = unsafe { rust_pci_get_device(dev) };
        ret
    }
    pub fn pci_get_subvendor(&self) -> u32 {
        e1000_println!();
        let dev: *mut device = self.inner.as_ptr();
        let ret = unsafe { rust_pci_get_subvendor(dev) };
        ret
    }
    pub fn pci_get_subdevice(&self) -> u32 {
        e1000_println!();
        let dev: *mut device = self.inner.as_ptr();
        let ret = unsafe { rust_pci_get_subdevice(dev) };
        ret
    }

    pub fn bus_alloc_resource_any(
        &self,
        systype: i32,
        rid: &mut i32,
        flags: u32,
    ) -> Option<Resource> {
        e1000_println!();
        let dev = self.inner.as_ptr() as *mut kernel::sys::bus_sys::device;
        let res: *mut resource = unsafe { bus_alloc_resource(dev, systype, rid, 0, !0, 1, flags) };
        if let Some(r) = NonNull::new(res) {
            Some(Resource {
                inner: r,
                dev: unsafe { NonNull::new_unchecked(dev as *mut kernel::sys::iflib_sys::device) },
                systype: systype,
                rid: *rid,
            })
        } else {
            None
        }
    }
}
impl Drop for PciDevice {
    fn drop(&mut self) {
        e1000_println!("inner: {:?}", self.inner);
    }
}
