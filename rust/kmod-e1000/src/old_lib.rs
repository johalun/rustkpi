#![feature(const_fn, plugin, used, global_asm, rustc_private)]
#![plugin(interpolate_idents)]
#![no_std]
#![feature(custom_attribute)]
#![allow(unused_imports, unused_macros)]

#[macro_use]
extern crate kernel;
#[macro_use]
extern crate lazy_static;

use kernel::sys::*;
// use kernel::sync::Arc;
// use kernel::sync::Mutex;


#[Macro_export]
macro_rules! devmethod {
    ($name:ident, $func:ident) => {
        kernel::sys::kobj_sys::kobj_method_t {
            desc: &kernel::sys::iflib_sys::$name_desc,
            func: $func as kernel::sys::kobj_sys::kobjop_t,
        },
    }
}

// #[Macro_export]
// macro_rules! driver_module_ordered {
//     ($name:ident, $busname:ident, $driver:ident, $devclass:ident, $evh:ident, $arg:ident, $order:path) => {
//         early_driver_module_ordered!($name, $busname, $driver, $devclass, $evh, $arg,
//                                      $order, kernel::sys::iflib_sys::BUS_PASS_DEFAULT);
//     }
// }


#[Macro_export]
macro_rules! driver_module {
    ($name:ident, $busname:tt, $driver:ident, $devclass:ident, $evh:ident, $arg:expr) => {
        early_driver_module!($name, $busname, $driver, $devclass, $evh, $arg,
                             kernel::sys::iflib_sys::BUS_PASS_DEFAULT);
    }
}

#[Macro_export]
macro_rules! early_driver_module {
    ($name:ident, $busname:tt, $driver:ident, $devclass:ident, $evh:ident, $arg:expr, $pass:path) => {
        early_driver_module_ordered!($name, $busname, $driver, $devclass, $evh, $arg,
                                     kernel::sys::kernel_sys::sysinit_elem_order::SI_ORDER_MIDDLE, $pass);
    }
}

#[Macro_export]
macro_rules! early_driver_module_ordered {
    ($name:ident, $busname:tt, $driver:ident, $devclass:ident, $evh:ident, $arg:expr, $order:path, $pass:path) => {

        // int	driver_module_handler(struct module *, int, void *);
        extern {
            fn driver_module_handler(module: *mut kernel::sys::module_sys::module, event: c_int, arg: *mut c_void) -> c_int;
        }

        interpolate_idents! {
            pub static [$name _ $busname _driver_mod]: kernel::sys::iflib_sys::driver_module_data =
                kernel::sys::iflib_sys::driver_module_data {
                    dmd_chainevh: $evh,
                    dmd_chainarg: $arg,
                    dmd_busname: cstr!(stringify!($busname)),
                    dmd_driver: &mut $driver as *mut kernel::sys::iflib_sys::kobj_class,
                    dmd_devclass: unsafe { &mut $devclass },
                    dmd_pass: $pass as i32,
                };
            pub static [$name _ $busname _mod]: kernel::sys::module_sys::moduledata_t =
                kernel::sys::module_sys::moduledata_t {
                    name: cstr!(stringify!([$busname / $name])),
                    evhand: Some(driver_module_handler),
                    priv_: &mut [$name _ $busname _driver_mod] as *mut _ as *mut c_void,
                };
            declare_module!([$name _ $busname], [$name _ $busname _mod],
                            kernel::sys::kernel_sys::sysinit_sub_id::SI_SUB_DRIVERS, $order);
        }
    }
}




//////////////////////////////////////////////////////////////////////////////////////////////////////

use kernel::sys::iflib_sys;


#[repr(C)]
// #[derive(Debug, Copy, Clone)]
pub struct module {
    _unused: [u8; 0],
}


#[repr(C)]
// #[derive(Debug, Copy)]
pub struct kobj_class {
    pub name: *const ::kernel::sys::raw::c_char,
    pub methods: *mut kernel::sys::iflib_sys::kobj_method_t,
    pub size: usize,
    pub baseclasses: *mut kernel::sys::iflib_sys::kobj_class_t,
    pub refs: c_uint,
    pub ops: kernel::sys::iflib_sys::kobj_ops_t,
}
pub type kobj_class_t = *mut kobj_class;


#[repr(C)]
// #[derive(Debug, Copy)]
pub struct driver_module_data {
    pub dmd_chainevh:
        ::core::option::Option<
            unsafe extern "C" fn(arg1: *mut kernel::sys::iflib_sys::module,
                                 arg2: ::kernel::sys::raw::c_int,
                                 arg3: *mut ::kernel::sys::raw::c_void)
                                 -> ::kernel::sys::raw::c_int,
        >,
    pub dmd_chainarg: *mut ::kernel::sys::raw::c_void,
    pub dmd_busname: *const ::kernel::sys::raw::c_char,
    pub dmd_driver: kobj_class_t,
    pub dmd_devclass: *mut kernel::sys::iflib_sys::devclass_t,
    pub dmd_pass: ::kernel::sys::raw::c_int,
}


pub struct adapter {
    id: i32,
}

extern "C" {
    fn driver_module_handler(
        module: *mut kernel::sys::module_sys::module,
        event: c_int,
        arg: *mut c_void,
    ) -> c_int;
}

// pub struct kobj_method {
//     pub desc: kobjop_desc_t,
//     pub func: kobjop_t,
// }
// pub type kobj_method_t = kobj_method;

// extern "C" {
//     #[link_name = "device_register_desc"]
//     pub static mut device_register_desc: kernel::sys::iflib_sys::kobjop_desc;
// }

/////////////////////////////////////////////////////////////////////////////////////////////////////////


// 0
#[no_mangle]
pub extern "C" fn rem_register_default() -> c_int {
    println!("rem_register_default");
    return 0;
}

// 1
#[no_mangle]
pub extern "C" fn rem_register() -> c_int {
    println!("rem_register");
    return 0;
}

// C
// typedef int			(*kobjop_t)(void);
// RUST
// pub type kobjop_t = Option<unsafe extern "C" fn() -> c_int>;
//
//
// C
// struct kobjop_desc {
// 	unsigned int	id;		/* unique ID */
// 	kobj_method_t	deflt;		/* default implementation */
// };
// RUST
// pub struct kobjop_desc {
//     pub id: ::kernel::sys::raw::c_uint,
//     pub deflt: kobj_method_t,
// }
// pub type kobjop_desc_t = *mut kobjop_desc;
//
//
// C
// struct kobj_method {
// 	kobjop_desc_t	desc;
// 	kobjop_t	func;
// };
// RUST
// pub struct kobj_method {
//     pub desc: kobjop_desc_t,
//     pub func: kobjop_t,
// }
// pub type kobj_method_t = kobj_method;




// pub static mut drd: *mut kernel::sys::iflib_sys::kobjop_desc = &mut device_register_desc;

// pub static drd: *const kernel::sys::iflib_sys::kobjop_desc = ;

// 2
pub static rem_methods: [kernel::sys::iflib_sys::kobj_method_t; 2] =
    [
        kernel::sys::iflib_sys::kobj_method_t {
            desc: &kernel::sys::iflib_sys::kobjop_desc {
                id: 1,
                deflt: kernel::sys::iflib_sys::kobj_method_t {
                    desc: 0 as *mut kernel::sys::iflib_sys::kobjop_desc,
                    func: Some(rem_register_default),
                },
            },
            func: Some(rem_register),
        },
        kernel::sys::iflib_sys::kobj_method_t {
            desc: kernel::ptr::null_mut(),
            func: None,
        },
    ];

// 3
pub static rem_driver: kernel::sys::iflib_sys::kobj_class = kernel::sys::iflib_sys::kobj_class {
    name: cstr!("rem"),
    methods: unsafe { &rem_methods as *const _ as *const kernel::sys::iflib_sys::kobj_method_t },
    size: 4, //kernel::mem::size_of<struct adapter>(),
    baseclasses: 0 as *mut *const kernel::sys::iflib_sys::kobj_class,
    refs: 0,
    ops: 0 as *mut kernel::sys::iflib_sys::kobj_ops,
};

struct devclass {
	TAILQ_ENTRY(devclass) link;
	devclass_t	parent;		/* parent in devclass hierarchy */
	driver_list_t	drivers;     /* bus devclasses store drivers for bus */
	char		*name;
	device_t	*devices;	/* array of devices indexed by unit */
	int		maxunit;	/* size of devices array */
	int		flags;
#define DC_HAS_CHILDREN		1

	struct sysctl_ctx_list sysctl_ctx;
	struct sysctl_oid *sysctl_tree;
};

// 4
pub static rem_devclass: kernel::sys::iflib_sys::devclass_t =
    &kernel::sys::iflib_sys::devclass { _unused: [] };

// driver_module!(rem, pci, rem_driver, rem_devclass, None, 0 as *mut c_void);
// Given by macro above: (5 and 6)
// 5
pub static rem_pci_driver_mod: kernel::sys::iflib_sys::driver_module_data =
    kernel::sys::iflib_sys::driver_module_data {
        dmd_chainevh: None,
        dmd_chainarg: 0 as *mut c_void,
        dmd_busname: cstr!("pci"),
        dmd_driver: &rem_driver as *const kernel::sys::iflib_sys::kobj_class,
        dmd_devclass: unsafe { &rem_devclass },
        dmd_pass: kernel::sys::iflib_sys::BUS_PASS_DEFAULT as i32,
    };

// 6
pub static rem_pci_mod: kernel::sys::module_sys::moduledata_t =
    kernel::sys::module_sys::moduledata_t {
        name: cstr!("pci/rem"),
        evhand: Some(driver_module_handler), // extern
        priv_: &rem_pci_driver_mod as *const _ as *const c_void,
    };





declare_module!(
    rem_pci,
    rem_pci_mod,
    kernel::sys::kernel_sys::sysinit_sub_id::SI_SUB_DRIVERS,
    kernel::sys::kernel_sys::sysinit_elem_order::SI_ORDER_MIDDLE
);

module_depend!(rem, pci, 1, 1, 1);
module_depend!(rem, ether, 1, 1, 1);
module_depend!(rem, iflib, 1, 1, 1);
module_depend!(rem, rustkpi, 1, 1, 1);




// #define	EARLY_DRIVER_MODULE_ORDERED(name, busname, driver, devclass,	\
//     evh, arg, order, pass)						\
// 									\
// static struct driver_module_data name##_##busname##_driver_mod = {	\
// 	evh, arg,							\
// 	#busname,							\
// 	(kobj_class_t) &driver,						\
// 	&devclass,							\
// 	pass								\
// };									\
// 									\
// static moduledata_t name##_##busname##_mod = {				\
// 	#busname "/" #name,						\
// 	driver_module_handler,						\
// 	&name##_##busname##_driver_mod					\
// };									\
// DECLARE_MODULE(name##_##busname, name##_##busname##_mod,		\
// 	       SI_SUB_DRIVERS, order)

// #define	EARLY_DRIVER_MODULE(name, busname, driver, devclass, evh, arg, pass) \
// 	EARLY_DRIVER_MODULE_ORDERED(name, busname, driver, devclass,	\
// 	    evh, arg, SI_ORDER_MIDDLE, pass)

// #define	DRIVER_MODULE_ORDERED(name, busname, driver, devclass, evh, arg,\
//     order)								\
// 	EARLY_DRIVER_MODULE_ORDERED(name, busname, driver, devclass,	\
// 	    evh, arg, order, BUS_PASS_DEFAULT)

// #define	DRIVER_MODULE(name, busname, driver, devclass, evh, arg)	\
// 	EARLY_DRIVER_MODULE(name, busname, driver, devclass, evh, arg,	\
// 	    BUS_PASS_DEFAULT)

// em0@pci0:0:2:0: class=0x020000 card=0x10088086 chip=0x100f8086 rev=0x00 hdr=0x00
//     vendor     = 'Intel Corporation'
//     device     = '82545EM Gigabit Ethernet Controller (Copper)'
//     class      = network
//     subclass   = ethernet

// #define E1000_DEV_ID_82545EM_COPPER		0x100F
// static pci_vendor_info_t em_vendor_info_array[] =
// {
//     PVID(0x8086, E1000_DEV_ID_82545EM_COPPER, "Intel(R) PRO/1000 Network Connection"),
//     PVID_END
// }

// static device_method_t em_methods[] = {
// 	/* Device interface */
// 	DEVMETHOD(device_register, em_register),
// 	DEVMETHOD(device_probe, iflib_device_probe),
// 	DEVMETHOD(device_attach, iflib_device_attach),
// 	DEVMETHOD(device_detach, iflib_device_detach),
// 	DEVMETHOD(device_shutdown, iflib_device_shutdown),
// 	DEVMETHOD(device_suspend, iflib_device_suspend),
// 	DEVMETHOD(device_resume, iflib_device_resume),
// 	DEVMETHOD_END
// };

// #define DEVMETHOD KOBJMETHOD

// #define KOBJMETHOD(NAME, FUNC) \
// 	{ &NAME##_desc, (kobjop_t) (1 ? FUNC : (NAME##_t *)NULL) }

// #define device_method_t		kobj_method_t
// typedef const struct kobj_method kobj_method_t;

// struct kobj_method {
// 	kobjop_desc_t	desc;
// 	kobjop_t	func;
// };




// static driver_t em_driver = {
// 	"em", em_methods, sizeof(struct adapter),
// };

// static devclass_t em_devclass;
// DRIVER_MODULE(em, pci, em_driver, em_devclass, 0, 0);

// MODULE_DEPEND(em, pci, 1, 1, 1);
// MODULE_DEPEND(em, ether, 1, 1, 1);
// MODULE_DEPEND(em, iflib, 1, 1, 1);

// driver_t:
// pub struct kobj_class {
//     pub name: *const ::kernel::sys::raw::c_char,
//     pub methods: *mut kobj_method_t,
//     pub size: usize,
//     pub baseclasses: *mut kobj_class_t,
//     pub refs: u_int,
//     pub ops: kobj_ops_t,
// }

// pub extern "C" fn module_event(_module: module_t, event: c_int, _arg: *mut c_void) -> c_int {
//     match event {
//         x if x == modeventtype_t::MOD_LOAD as c_int => {
//             println!("Got kernel module event: LOAD");
//         }
//         x if x == modeventtype_t::MOD_UNLOAD as c_int => {
//             println!("Got kernel module event: UNLOAD");
//         }
//         x if x == modeventtype_t::MOD_QUIESCE as c_int => {}
//         x if x == modeventtype_t::MOD_SHUTDOWN as c_int => {}
//         _ => {}
//     }
//     0
// }

// pub static MODULE_DATA: moduledata_t = moduledata_t {
//     name: b"rem\0" as *const u8 as *const i8,
//     evhand: Some(module_event),
//     priv_: 0 as *mut c_void,
// };

// // These macros require interpolate_idents
// declare_module!(
//     rem,
//     MODULE_DATA,
//     sysinit_sub_id::SI_SUB_DRIVERS,
//     sysinit_elem_order::SI_ORDER_MIDDLE
// );
// module_depend!(test, rustkpi, 1, 1, 1);
