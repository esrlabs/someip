//! Crate for parsing the SOME/IP network protocol (without payload interpretation).

#![warn(missing_docs)]

mod error;
/// Parse someip messages
pub mod parser;
/// Serialize someip messages
pub mod serializer;
mod types;

pub use error::Error;
pub use types::*;
