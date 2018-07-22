//! If logging is not compiled into the library, this module defines the logging macros to result
//! in nothing.

/// This macro is defined if the `logging` feature is _not_ compiled into the library
///
/// It ignores all logging calls inside the library.
#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)*) => { };
    ($($arg:tt)*) => { };
}

/// This macro is defined if the `logging` feature is _not_ compiled into the library
///
/// It ignores all logging calls inside the library.
#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! error {
    (target: $target:expr, $($arg:tt)*) => { };
    ($($arg:tt)*) => { };
}

/// This macro is defined if the `logging` feature is _not_ compiled into the library
///
/// It ignores all logging calls inside the library.
#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! info {
    (target: $target:expr, $($arg:tt)*) => { };
    ($($arg:tt)*) => { };
}

/// This macro is defined if the `logging` feature is _not_ compiled into the library
///
/// It ignores all logging calls inside the library.
#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! log {
    (target: $target:expr, $($arg:tt)*) => { };
    ($($arg:tt)*) => { };
}

/// This macro is defined if the `logging` feature is _not_ compiled into the library
///
/// It ignores all logging calls inside the library.
#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)*) => { };
    ($($arg:tt)*) => { };
}

/// This macro is defined if the `logging` feature is _not_ compiled into the library
///
/// It ignores all logging calls inside the library.
#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! warn {
    (target: $target:expr, $($arg:tt)*) => { };
    ($($arg:tt)*) => { };
}

