//! Crate for parsing the SOME/IP network protocol (without payload interpretation).

#![warn(missing_docs)]

mod error;
/// Parse someip messages
mod parser;
/// Serialize someip messages
mod serializer;
/// Message types
mod types;

pub use error::Error;
pub use types::*;
