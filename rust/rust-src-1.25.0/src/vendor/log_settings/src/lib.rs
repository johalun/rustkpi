//! This crate enables libraries that use the `log` crate (or an equivalent) to communicate with
//! the actual logger, without requiring the library to know about the type of logger that is used.
//! The crate
//!
//! # On the library side
//!
//! You can set a value by accessing the `Settings` struct through the `settings` function.
//!
//! ```rust
//! extern crate log_settings;
//! log_settings::settings().indentation += 1;
//! ```
//!
//! # On the executable side
//!
//! You can read a value by accessing the `Settings` struct through the `settings` function.
//!
//! ```rust
//! #[macro_use] extern crate log;
//! extern crate env_logger;
//! extern crate log_settings;
//!
//! use std::env;
//! use log::{LogRecord, LogLevelFilter};
//! use env_logger::LogBuilder;
//!
//! fn main() {
//!     let format = |record: &LogRecord| {
//!         // prepend spaces to indent the final string
//!         let indentation = log_settings::settings().indentation;
//!         let spaces = "                                  ";
//!         let indentation = s[..std::cmp::max(indentation, spaces.len())];
//!         format!("{}{} - {}", indentation, record.level(), record.args())
//!     };
//!
//!     let mut builder = LogBuilder::new();
//!     builder.format(format).filter(None, LogLevelFilter::Info);
//!
//!     if env::var("RUST_LOG").is_ok() {
//!        builder.parse(&env::var("RUST_LOG").unwrap());
//!     }
//!
//!     builder.init().unwrap();
//! }
//! ```

#[macro_use] extern crate lazy_static;

use std::sync::{Mutex, MutexGuard};

lazy_static! {
    static ref SETTINGS: Mutex<Settings> = Mutex::new(Settings{
        indentation: 0,
        __dont_match_on_this: (),
    });
}

/// Contains various settings that libraries might want to set to notify loggers that also use this
/// crate of internal library states
pub struct Settings {
    /// sets the indentation level of the log messages
    pub indentation: usize,
    // prevents users from destructuring or creating a Settings struct
    __dont_match_on_this: (),
}

/// obtains a handle to the internal settings struct
pub fn settings<'a>() -> MutexGuard<'a, Settings> {
    SETTINGS.lock().expect("the global setting mutex has been poisoned")
}
