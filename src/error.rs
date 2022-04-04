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
        /// The actual size received.
        /// For strings this is in bytes for sequences in elements.
        actual: usize,
    },
    /// Invalid length field value
    #[error("Invalid length field: {0}")]
    InvalidLengthField(u32),
    /// Invalid return code value
    #[error("Unknown return code: {0}")]
    InvalidReturnCode(u8),
    /// Invalid message type value
    #[error("Unknown message type value: {0}")]
    InvalidMessageType(u8),
    /// Unknown sd entry value
    #[error("Unknown sd entry value: {0}")]
    UnknownSdEntry(u8),
    /// Unknown sd option value
    #[error("Unknown sd option value: {0}")]
    UnknownSdOption(u8),
    /// Invalid ip proto value
    #[error("Unknown ip proto value: {0}")]
    InvalidIpProto(u8),
    /// Invalid ip proto value
    #[cfg(feature = "url")]
    #[error("Invalid url: {0}")]
    InvalidUrl(&'static str),
}

/// Transforms std::io::Error to a Error.
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}
