use std::io;
use thiserror::Error;

/// Custom Error type
#[derive(Error, Debug)]
pub enum Error {
    /// IO Error
    #[error("IO Error: {0}")]
    Io(io::Error),
    /// A string or sequence with a minimum size (min_size for strings and min_elements for sequences)
    /// was deserialized and contained less data than the minimum size.
    #[error("Not enough data: min: {min}, actual: {actual}")]
    NotEnoughData {
        /// The minimum size required.
        /// For strings this is in bytes for sequences in elements.
        min: usize,
        ///The actual size recived.
        /// For strings this is in bytes for sequences in elements.
        actual: usize,
    },
    /// Unknown return code value
    #[error("Uknown return code: {0}")]
    UnknownReturnCode(u8),
    /// Unknown message type value
    #[error("Uknown message type value: {0}")]
    UnknownMessageType(u8),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}
