#![recursion_limit = "1024"]
// We need this for error_chain, unfortunately.

/// # toml-query
///
/// A crate to help executing queries on toml data structures inside Rust code.
///

// external crates

#[macro_use] extern crate error_chain;
#[macro_use] extern crate is_match;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate toml;

#[cfg(feature = "log")]
#[macro_use] extern crate log;

#[cfg(test)]
#[macro_use] extern crate quickcheck;

// public modules

#[cfg(not(feature = "log"))]
#[macro_use] pub mod log;

pub mod error;
pub mod read;
pub mod set;
pub mod insert;
pub mod delete;
pub mod value;
mod util;

// private modules

mod tokenizer;
mod resolver;

