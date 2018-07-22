// #define E1000_REGISTER(hw, reg) (((hw)->mac.type >= e1000_82543) \
//     ? reg : e1000_translate_register_82542(reg))

// #define E1000_READ_REG(hw, reg) \
//     bus_space_read_4(((struct e1000_osdep *)(hw)->back)->mem_bus_space_tag, \
//         ((struct e1000_osdep *)(hw)->back)->mem_bus_space_handle, \
//         E1000_REGISTER(hw, reg))

// #[macro_export]
// macro_rules! e1000_register {
//     ($mactype:ident, $reg:expr) => {
//         match $mactype {
//             x if x >= sys::e1000::MacType::82543 => $reg,
//             _ => e1000_82542::e1000_translate_register($reg)
//         }
//     }
// }

// #[macro_export]
// macro_rules! e1000_read_reg {
//     ($mactype:ident, $reg:expr) => {
//     }
// }

// #[macro_export]
// macro_rules! set_mac_ops {
//     ($fn:ident, $module:ident) => {
//         adapter.hw.mac.ops.$fn = Some($module::$fn)
//     }
// }

#[macro_export]
macro_rules! e1000_println {
    () => (
        if ::DEBUG_PRINT {
            println!("DEBUG => {}", function_path!());
        }
    );
    ($fmt:expr) => (
        if ::DEBUG_PRINT {
            println!("DEBUG => {}: {}", function_path!(), format_args!($fmt));
        }
    );
    ($fmt:expr, $($arg:tt)*) => (
        if ::DEBUG_PRINT {
            println!("DEBUG => {}: {}", function_path!(), format_args!($fmt, $($arg)*));
        }
    );
}

#[macro_export]
macro_rules! e1000_verbose_println {
    () => (
        if ::DEBUG_VERBOSE_PRINT {
            println!("DEBUG => {}", function_path!());
        }
    );
    ($fmt:expr) => (
        if ::DEBUG_VERBOSE_PRINT {
            println!("DEBUG => {}: {}", function_path!(), format_args!($fmt));
        }
    );
    ($fmt:expr, $($arg:tt)*) => (
        if ::DEBUG_VERBOSE_PRINT {
            println!("DEBUG => {}: {}", function_path!(), format_args!($fmt, $($arg)*));
        }
    );
}

#[macro_export]
macro_rules! e1000_phy_println {
    () => (
        if ::DEBUG_PHY_PRINT {
            println!("DEBUG PHY => {}", function_path!());
        }
    );
    ($fmt:expr) => (
        if ::DEBUG_PHY_PRINT {
            println!("DEBUG PHY => {}: {}", function_path!(), format_args!($fmt));
        }
    );
    ($fmt:expr, $($arg:tt)*) => (
        if ::DEBUG_PHY_PRINT {
            println!("DEBUG PHY => {}: {}", function_path!(), format_args!($fmt, $($arg)*));
        }
    );
}

#[macro_export]
macro_rules! e1000_mac_println {
    () => (
        if ::DEBUG_MAC_PRINT {
            println!("DEBUG MAC => {}", function_path!());
        }
    );
    ($fmt:expr) => (
        if ::DEBUG_MAC_PRINT {
            println!("DEBUG MAC => {}: {}", function_path!(), format_args!($fmt));
        }
    );
    ($fmt:expr, $($arg:tt)*) => (
        if ::DEBUG_MAC_PRINT {
            println!("DEBUG MAC => {}: {}", function_path!(), format_args!($fmt, $($arg)*));
        }
    );
}

#[macro_export]
macro_rules! eprintln {
    ($str:ident) => (
        println!("=> ERROR => {}: {}", function_path!(), $str);
    );
    ($fmt:expr) => (
        println!("=> ERROR => {}: {}", function_path!(), format_args!($fmt));
    );
    ($fmt:expr, $($arg:tt)*) => (
        println!("=> ERROR => {}: {}", function_path!(), format_args!($fmt, $($arg)*));
    );
}

#[macro_export]
macro_rules! incomplete {
    () => {
        println!("INCOMPLETE => {}", function_path!());
        println!("============> {}:{}:{}", file!(), line!(), column!());
    }
}

#[macro_export]
macro_rules! incomplete_return {
    () => {
        incomplete!();
        return Err("incomplete function - returning error".to_string());
    }
}

#[macro_export]
macro_rules! unsupported {
    () => {
        println!("==> UNSUPPORTED HARDWARE => {}", function_path!());
        println!("==> {}:{}:{}", file!(), line!(), column!());
    }
}

#[macro_export]
macro_rules! function_path {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            extern crate core;
            unsafe { core::intrinsics::type_name::<T>() }
        }
        let name = type_name_of(f);
        &name[6..name.len() - 4]
    }}
}

#[macro_export]
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(0 as *const $ty)).$field as *const _ as usize
    }
}

#[macro_export]
macro_rules! htons {
    ($v:expr) => {
        ((($v as u16) << 8) | (($v as u16) >> 8))
    }
}

#[macro_export]
macro_rules! htole64 {
    ($v:expr) => {
        ($v as u64)
    }
}

#[macro_export]
macro_rules! htole32 {
    ($v:expr) => {
        ($v as u32)
    }
}

#[macro_export]
macro_rules! le32toh {
    ($v:expr) => {
        ($v as u32)
    }
}

#[macro_export]
macro_rules! le16toh {
    ($v:expr) => {
        ($v as u16)
    }
}

#[macro_export]
macro_rules! btst {
    ($a:expr, $b:expr) => {
        (($a) & ($b)) != 0
    }
}

// #define E1000_DIVIDE_ROUND_UP(a, b)	(((a) + (b) - 1) / (b)) /* ceil(a/b) */

#[macro_export]
macro_rules! divide_round_up {
    ($a:expr, $b:expr) => {
        ((($a) + ($b) - 1) / ($b))
    }
}
