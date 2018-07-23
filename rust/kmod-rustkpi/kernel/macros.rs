#![allow(unused_macros)]


// Rusty macros

#[macro_export]
macro_rules! cstr {
    ($s:expr) => (
        concat!($s, "\0") as *const str as *const [::kernel::sys::raw::c_char]
            as *const ::kernel::sys::raw::c_char
    )
}

#[macro_export]
macro_rules! print {
    ($fmt:expr) => ({
        use kernel::fmt::Write;
	use kernel::io::KernelDebugWriter;
	let mut writer = KernelDebugWriter {};
        writer.write_fmt(format_args!($fmt)).unwrap();
    });

    // Dynamic implementation that processes format arguments
    ($fmt:expr, $($arg:tt)*) => ({
	use kernel::fmt::Write;
	use kernel::io::KernelDebugWriter;
	let mut writer = KernelDebugWriter {};
        writer.write_fmt(format_args!($fmt, $($arg)*)).unwrap();
    });
}

#[macro_export]
macro_rules! println {
    ($fmt:expr)              => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)+) => (print!(concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => (if cfg!(debug_assertions) { print!($($arg)*) })
}

#[macro_export]
macro_rules! debugln {
    ($($arg:tt)*) => (if cfg!(debug_assertions) { println!($($arg)*) })
}



// Kernel C macros converted to Rust


#[macro_export]
macro_rules! roundup {
    ($x:expr, $y:expr) => (
        ((($x) + (($y) - 1)) / ($y)) * ($y)
    )
}

#[macro_export]
macro_rules! roundup2 {
    ($x:expr, $y:expr) => (
        (($x) + (($y) - 1)) & (!(($y) - 1))
    )
}

#[macro_export]
macro_rules! module_kernel_maxver {
    () => (
        roundup!(kernel::sys::param_sys::__FreeBSD_version as usize, 100000) - 1
    )
}

#[macro_export]
macro_rules! data_set {
    ($set:ident, $sym:ident, $type:path, $section:tt) => {
        global_asm!(concat!(".globl __start_set_", stringify!($set)));
        global_asm!(concat!(".globl  __stop_set_", stringify!($set)));
        interpolate_idents! {
            #[used]
            #[link_section = $section]
            #[allow(non_camel_case_types, non_upper_case_globals)]
            pub static [__set_ $set _sym_ $sym]: &$type = &($sym);
        }
    }
}

#[macro_export]
macro_rules! sysinit {
    ($uniquifier:tt, $subsystem:path, $order:path, $func:path, $ident:ident) => (
        interpolate_idents! {
            #[allow(non_camel_case_types, non_upper_case_globals)]
            pub static [$uniquifier _sys_init]: kernel::sys::kernel_sys::sysinit =
                kernel::sys::kernel_sys::sysinit {
                    subsystem: $subsystem,
                    order: $order,
                    func: Some($func),
                    udata: &$ident as *const _ as *const kernel::sys::c_void,
                };
            data_set!(sysinit_set, [$uniquifier _sys_init], kernel::sys::kernel_sys::sysinit,
                      "set_sysinit_set");
        }
    )
}

#[macro_export]
macro_rules! module_metadata {
    ($uniquifier:tt, $type:expr, $data:ident, $cval:tt) => (
        interpolate_idents! {
            #[allow(non_camel_case_types, non_upper_case_globals)]
            pub static [_mod_metadata $uniquifier]: kernel::sys::module_sys::mod_metadata =
                kernel::sys::module_sys::mod_metadata {
                    md_version: kernel::sys::module_sys::MDT_STRUCT_VERSION as i32,
                    md_type: $type,
                    md_cval: concat!(stringify!($cval), "\0") as *const _ as *const i8,
                    md_data: &$data as *const _ as *const kernel::sys::c_void,
                };
            data_set!(modmetadata_set, [_mod_metadata $uniquifier],
                      kernel::sys::module_sys::mod_metadata,
                      "set_modmetadata_set");
        }
    );
}

#[macro_export]
macro_rules! module_depend {
    ($module:tt, $mdepend:tt, $vmin:expr, $vpref:expr, $vmax:expr) => (
        interpolate_idents! {
            #[link_section = ".data"]
            #[allow(non_camel_case_types, non_upper_case_globals)]
            pub static [_ $module _depend_on_ $mdepend]: kernel::sys::module_sys::mod_depend =
                kernel::sys::module_sys::mod_depend {
                    md_ver_minimum: $vmin,
                    md_ver_preferred: $vpref,
                    md_ver_maximum: $vmax,
                };
            module_metadata!([_md_ $module _on_ $mdepend],
                             kernel::sys::module_sys::MDT_DEPEND as i32,
                             [_ $module _depend_on_ $mdepend], $mdepend);
        }
    );
}

#[macro_export]
macro_rules! module_version {
    ($module:tt, $ver:expr) => (
        interpolate_idents! {
            #[link_section = ".data"]
            #[allow(non_camel_case_types, non_upper_case_globals)]
            pub static [_ $module _version]: kernel::sys::module_sys::mod_version =
                kernel::sys::module_sys::mod_version {
                    mv_version: $ver,
                };
            module_metadata!([_ $module _version],
                             kernel::sys::module_sys::MDT_VERSION as i32,
                             [_ $module _version], $module);
        }
    );
}

#[macro_export]
macro_rules! declare_module {
    ($name:tt, $data:ident, $sub:path, $order:path) => (
        declare_module!($name, $data, $sub, $order, module_kernel_maxver!());
    );
    ($name:tt, $data:ident, $sub:path, $order:path, $maxver:expr) => (
        interpolate_idents! {
            module_depend!($name, kernel, kernel::sys::param_sys::__FreeBSD_version as i32,
                           kernel::sys::param_sys::__FreeBSD_version as i32, $maxver as i32);
            module_metadata!([_md_ $name], kernel::sys::module_sys::MDT_MODULE as i32,
                             $data, $name);
            sysinit!([$name module], $sub, $order,
                     kernel::sys::module_sys::module_register_init, $data);
        }
    )
}

#[macro_export]
macro_rules! malloc_define {
    ($type:tt, $shortdesc:tt, $longdesc:tt) => (
        #[allow(non_upper_case_globals)]
        pub static $type: kernel::sys::malloc_type = kernel::sys::malloc_sys::malloc_type {
            ks_next: 0 as *mut _ as *mut kernel::sys::malloc_sys::malloc_type,
            ks_magic: kernel::sys::malloc_sys::M_MAGIC,
            ks_shortdesc: concat!(stringify!($shortdesc), "\0") as *const _ as *const i8,
            ks_handle: 0 as *mut _ as *mut kernel::sys::malloc_sys::malloc_type,
        };
        interpolate_idents! {
            sysinit!([$type _init], kernel::sys::kernel_sys::sysinit_sub_id::SI_SUB_KMEM,
                     kernel::sys::kernel_sys::sysinit_elem_order::SI_ORDER_THIRD, malloc_init, $type);
            sysuninit!([$type _uninit], kernel::sys::kernel_sys::sysinit_sub_id::SI_SUB_KMEM,
                       kernel::sys::kernel_sys::sysinit_sub_id::SI_ORDER_ANY, malloc_uninit, $type);
        }
    )
}

#[macro_export]
macro_rules! sysctl_node {
    ($parent:ident, $nbr:path, $name:ident, $access:path, $handler:path, $descr:pat) => {
        sysctl_node_with_label!($parent, $nbr, $name, $access, $handler,
                                $descr, 0usize);
    }
}

#[macro_export]
macro_rules! sysctl_node_with_label {
    ($parent:ident, $nbr:path, $name:ident, $access:path, $handler:expr, $descr:pat, $label:pat) => {
        sysctl_oid_global!($parent, $nbr, $name, kernel::sys::sysctl_sys::CTLTYPE_NODE|($access),
                           0usize, 0usize, $handler, "N", $descr, $label);
    }
}


#[macro_export]
macro_rules! sysctl_children {
    ($parent:ident) => {
        &$parent
    }
}

#[macro_export]
macro_rules! sysctl_oid_global {
    ($parent:ident, $nbr:path, $name:ident, $kind:expr, $a1:expr, $a2:expr, $handler:expr,
     $fmt:tt, $descr:pat, $label:pat) => {
        interpolate_idents! {
            sysctl_oid_raw!([sysctl__ $parent _ $name],
                            sysctl_children!([sysctl__ $parent]),
                            $nbr, $name, $kind, $a1, $a2, $handler, $fmt, $descr, $label);
        }
    }
}

#[macro_export]
macro_rules! sysctl_oid_raw {
    ($id:tt, $parent_child_head:expr, $nbr:path, $name:ident,
     $kind:expr, $a1:expr, $a2:expr, $handler:expr, $fmt:tt,
     $descr:pat, $label:pat) => {
        interpolate_idents! {
            pub static $id: kernel::sys::sysctl_sys::sysctl_oid = unsafe { kernel::sys::sysctl_sys::sysctl_oid {

                oid_link: kernel::sys::sysctl_sys::sysctl_oid__bindgen_ty_1 { sle_next: 0 as *const kernel::sys::sysctl_sys::sysctl_oid },
                oid_children: kernel::sys::sysctl_sys::sysctl_oid_list { slh_first: 0 as *const kernel::sys::sysctl_sys::sysctl_oid },
                oid_parent: $parent_child_head as *const kernel::sys::sysctl_sys::sysctl_oid,
                oid_refcnt: 0,
                oid_running: 0,
                oid_number: $nbr as kernel::sys::c_int,
                oid_kind: $kind as kernel::sys::kernel_sys::u_int,
                oid_arg1: &$a1 as *const _ as *const kernel::sys::c_void,
                oid_arg2: $a2 as kernel::sys::kernel_sys::intmax_t,
                oid_name: concat!(stringify!($name), "\0") as *const _ as *const kernel::sys::c_char,
                oid_handler: $handler,
                oid_fmt: concat!(stringify!($fmt), "\0") as *const _ as *const kernel::sys::c_char,
                oid_descr: concat!(stringify!($descr), "\0") as *const _ as *const kernel::sys::c_char,
                oid_label: concat!(stringify!($label), "\0") as *const _ as *const kernel::sys::c_char,

            }};
        }
        data_set!(sysctl_set, $id, kernel::sys::sysctl_sys::sysctl_oid, "set_sysctl_set");
    }
}

#[macro_export]
macro_rules! sysctl_int {
    ($parent:ident, $nbr:path, $name:ident, $access:path, $ptr:expr, $val:expr, $descr:pat) => {
        sysctl_int_with_label!($parent, $nbr, $name, $access, $ptr, $val, $descr, 0);
    }
}

#[macro_export]
macro_rules! sysctl_int_with_label {
    ($parent:ident, $nbr:path, $name:ident, $access:path, $ptr:expr, $val:expr, $descr:pat, $label:pat) => {
        sysctl_oid_with_label!($parent, $nbr, $name,
                               kernel::sys::sysctl_sys::CTLTYPE_INT|
                               kernel::sys::sysctl_sys::CTLFLAG_MPSAFE|
                               ($access),
                               $ptr, $val, Some(sysctl_handle_int), "I", $descr, $label);
    }
}

#[macro_export]
macro_rules! sysctl_oid_with_label {
    ($parent:ident, $nbr:path, $name:ident, $kind:expr, $a1:expr, $a2:expr, $handler:expr, $fmt:tt, $descr:pat, $label:pat) => {
        interpolate_idents! {
            sysctl_oid_raw!([sysctl__ $parent _ $name],
                            sysctl_children!([sysctl__ $parent]),
                            $nbr, $name, $kind, $a1, $a2, $handler, $fmt, $descr, $label);
        }
    }
}
