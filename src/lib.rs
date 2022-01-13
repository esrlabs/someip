//! A Rust library for parsing the SOME/IP network protocol (without payload interpretation).
//!
mod error;
pub mod parser;
pub mod serializer;
mod types;

pub use error::Error;
pub use types::*;
