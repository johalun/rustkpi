
#![no_std]
#![allow(unused_attributes)]
#![deny(warnings)]

#![rustc_alloc_kind = "lib"]
#![unstable(feature = "alloc_system",
            reason = "this library is unlikely to be stabilized in its current \
                      form or name",
            issue = "27783")]

#![feature(global_allocator)]
#![feature(allocator_api)]
#![feature(alloc)]
#![feature(staged_api)]
#![feature(rustc_attrs)]

// #![feature(core_intrinsics)]

#![allow(warnings)]


extern crate alloc;

use self::alloc::heap::{Alloc, AllocErr, Layout, Excess, CannotReallocInPlace};

#[unstable(feature = "allocator_api", issue = "32838")]
pub struct System;



#[unstable(feature = "allocator_api", issue = "32838")]
unsafe impl Alloc for System {
    #[inline]
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        (&*self).alloc(layout)
    }

    #[inline]
    unsafe fn alloc_zeroed(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        (&*self).alloc_zeroed(layout)
    }

    #[inline]
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        (&*self).dealloc(ptr, layout)
    }

    #[inline]
    unsafe fn realloc(
        &mut self,
        ptr: *mut u8,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<*mut u8, AllocErr> {
        (&*self).realloc(ptr, old_layout, new_layout)
    }

    fn oom(&mut self, err: AllocErr) -> ! {
        (&*self).oom(err)
    }

    #[inline]
    fn usable_size(&self, layout: &Layout) -> (usize, usize) {
        (&self).usable_size(layout)
    }

    #[inline]
    unsafe fn alloc_excess(&mut self, layout: Layout) -> Result<Excess, AllocErr> {
        (&*self).alloc_excess(layout)
    }

    #[inline]
    unsafe fn realloc_excess(
        &mut self,
        ptr: *mut u8,
        layout: Layout,
        new_layout: Layout,
    ) -> Result<Excess, AllocErr> {
        (&*self).realloc_excess(ptr, layout, new_layout)
    }

    #[inline]
    unsafe fn grow_in_place(
        &mut self,
        ptr: *mut u8,
        layout: Layout,
        new_layout: Layout,
    ) -> Result<(), CannotReallocInPlace> {
        (&*self).grow_in_place(ptr, layout, new_layout)
    }

    #[inline]
    unsafe fn shrink_in_place(
        &mut self,
        ptr: *mut u8,
        layout: Layout,
        new_layout: Layout,
    ) -> Result<(), CannotReallocInPlace> {
        (&*self).shrink_in_place(ptr, layout, new_layout)
    }
}

// Allocator for FreeBSD kernel
mod raw;
mod kern_malloc;
mod platform {


    use kern_malloc as kern;
    use raw;
    use System;
    use alloc::heap::{Alloc, AllocErr, Layout};

    #[unstable(feature = "allocator_api", issue = "32838")]
    unsafe impl<'a> Alloc for &'a System {
        #[inline]
        unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
            let ptr = kern::malloc(
                layout.size() as raw::c_size_t,
                &mut kern::M_DEVBUF[0],
                kern::M_NOWAIT as i32,
            ) as *mut u8;
            if !ptr.is_null() {
                Ok(ptr)
            } else {
                Err(AllocErr::Exhausted { request: layout })
            }
        }

        #[inline]
        unsafe fn alloc_zeroed(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
            let ptr = kern::malloc(
                layout.size() as raw::c_size_t,
                &mut kern::M_DEVBUF[0],
                kern::M_NOWAIT as i32 | kern::M_ZERO as i32,
            ) as *mut u8;
            if !ptr.is_null() {
                Ok(ptr)
            } else {
                Err(AllocErr::Exhausted { request: layout })
            }
        }

        #[inline]
        unsafe fn dealloc(&mut self, ptr: *mut u8, _layout: Layout) {
            kern::free(ptr as *mut raw::c_void, &mut kern::M_DEVBUF[0])
        }

        #[inline]
        unsafe fn realloc(
            &mut self,
            ptr: *mut u8,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<*mut u8, AllocErr> {
            if old_layout.align() != new_layout.align() {
                return Err(AllocErr::Unsupported {
                    details: "cannot change alignment on `realloc`",
                });
            }

            let ptr = kern::realloc(
                ptr as *mut raw::c_void,
                new_layout.size() as raw::c_size_t,
                &mut kern::M_DEVBUF[0],
                kern::M_NOWAIT as i32,
            ) as *mut u8;
            if !ptr.is_null() {
                Ok(ptr as *mut u8)
            } else {
                Err(AllocErr::Exhausted { request: new_layout })
            }
        }

        fn oom(&mut self, err: AllocErr) -> ! {
            use core::fmt::{self, Write};

            // Print a message to stderr before aborting to assist with
            // debugging. It is critical that this code does not allocate any
            // memory since we are in an OOM situation. Any errors are ignored
            // while printing since there's nothing we can do about them and we
            // are about to exit anyways.
            drop(writeln!(Stderr, "fatal runtime error: {}", err));
            unsafe { loop {} }

            struct Stderr;

            impl Write for Stderr {
                fn write_str(&mut self, s: &str) -> fmt::Result {
                    unsafe {
                        kern::printf(s.as_ptr() as *const i8);

                        // libc::write(
                        //     libc::STDERR_FILENO,
                        //     s.as_ptr() as *const libc::c_void,
                        //     s.len(),
                        // );
                    }
                    Ok(())
                }
            }
        }
    }
}
