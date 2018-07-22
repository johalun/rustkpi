
#[macro_export]
macro_rules! roundup {
    ($x:expr, $y:expr) => (
         (((($x) + (($y) - 1)) / ($y)) * ($y))
    )
}

#[macro_export]
macro_rules! module_kernel_maxver {
    () => (
        roundup!(kernel::kernel_sys::__FreeBSD_version as usize, 100000) - 1
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
            pub static [$uniquifier _sys_init]: kernel::kernel_sys::sysinit =
                kernel::kernel_sys::sysinit {
                    subsystem: $subsystem,
                    order: $order,
                    func: Some($func),
                    udata: &$ident as *const _ as *const std::os::raw::c_void,
                };
            data_set!(sysinit_set, [$uniquifier _sys_init], kernel::kernel_sys::sysinit,
                      "set_sysinit_set");
        }
    )
}

#[macro_export]
macro_rules! module_metadata {
    ($uniquifier:tt, $type:expr, $data:ident, $cval:tt) => (
        interpolate_idents! {
            #[allow(non_camel_case_types, non_upper_case_globals)]
            pub static [_mod_metadata $uniquifier]: kernel::module_sys::mod_metadata =
                kernel::module_sys::mod_metadata {
                    md_version: kernel::module_sys::MDT_STRUCT_VERSION as i32,
                    md_type: $type,
                    md_cval: concat!(stringify!($cval), "\0") as *const _ as *const i8,
                    md_data: &$data as *const _ as *const std::os::raw::c_void,
                };
            data_set!(modmetadata_set, [_mod_metadata $uniquifier], kernel::module_sys::mod_metadata,
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
            pub static [_ $module _depend_on_ $mdepend]: kernel::module_sys::mod_depend =
                kernel::module_sys::mod_depend {
                    md_ver_minimum: $vmin,
                    md_ver_preferred: $vpref,
                    md_ver_maximum: $vmax,
                };
            module_metadata!([_md_ $module _on_ $mdepend], kernel::module_sys::MDT_DEPEND as i32,
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
            pub static [_ $module _version]: kernel::module_sys::mod_version =
                kernel::module_sys::mod_version {
                    mv_version: $ver,
                };
            module_metadata!([_ $module _version], kernel::module_sys::MDT_VERSION as i32,
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
            module_depend!($name, kernel, kernel::conf_sys::__FreeBSD_version as i32,
                           kernel::conf_sys::__FreeBSD_version as i32, $maxver as i32);
            module_metadata!([_md_ $name], kernel::module_sys::MDT_MODULE as i32, $data, $name);
            sysinit!([$name module], $sub, $order, kernel::module_sys::module_register_init, $data);
        }
    )
}

#[macro_export]
macro_rules! malloc_define {
    ($type:tt, $shortdesc:tt, $longdesc:tt) => (
        #[allow(non_upper_case_globals)]
        pub static $type: kernel::malloc_sys::malloc_type = kernel::malloc_sys::malloc_type {
            ks_next: 0 as *mut _ as *mut kernel::malloc_sys::malloc_type,
            ks_magic: kernel::malloc_sys::M_MAGIC,
            ks_shortdesc: concat!(stringify!($shortdesc), "\0") as *const _ as *const i8,
            ks_handle: 0 as *mut _ as *mut kernel::malloc_sys::malloc_type,
        };
        interpolate_idents! {
            sysinit!([$type _init], kernel::kernel_sys::sysinit_sub_id::SI_SUB_KMEM,
                     kernel::kernel_sys::sysinit_elem_order::SI_ORDER_THIRD, malloc_init, $type);
            sysuninit!([$type _uninit], kernel::kernel_sys::sysinit_sub_id::SI_SUB_KMEM,
                       kernel::kernel_sys::sysinit_sub_id::SI_ORDER_ANY, malloc_uninit, $type);
        }
    )
}

// #[macro_export]
// macro_rules! malloc_declare {
//     ($type:tt) => (
//         extern "C" {
//             pub static $type: kernel::kernel_sys::malloc_type;
//         }
//     )
// }
