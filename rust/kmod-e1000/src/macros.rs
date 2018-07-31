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
        println!("ERROR => {}: {}", function_path!(), $str);
    );
    ($fmt:expr) => (
        println!("ERROR => {}: {}", function_path!(), format_args!($fmt));
    );
    ($fmt:expr, $($arg:tt)*) => (
        println!("ERROR => {}: {}", function_path!(), format_args!($fmt, $($arg)*));
    );
}

#[macro_export]
macro_rules! incomplete {
    () => {
        println!("INCOMPLETE FUNCTION => {}", function_path!());
        println!("=> {}:{}:{}", file!(), line!(), column!());
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
        println!("UNSUPPORTED HARDWARE => {}", function_path!());
        println!("=> {}:{}:{}", file!(), line!(), column!());
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

#[macro_export]
macro_rules! divide_round_up {
    ($a:expr, $b:expr) => {
        ((($a) + ($b) - 1) / ($b))
    }
}

// #ifdef INVARIANTS
// #define ASSERT_CTX_LOCK_HELD(hw) (sx_assert(iflib_ctx_lock_get(((struct e1000_osdep *)hw->back)->ctx), SX_XLOCKED))
// #else
// #define ASSERT_CTX_LOCK_HELD(hw)
// #endif
#[macro_export]
macro_rules! assert_ctx_lock_held {
    ($a:expr) => {
    }
}
