//! Error types.

use std::ffi::NulError;
use std::io;
use std::str::Utf8Error;

use thiserror::Error as ThisError;

/// Commonly encountered error types.
#[derive(ThisError, Debug)]
pub enum FfiError {
    /// An interior nul byte was found.
    #[error("Interior nul byte")]
    NulByte(#[from] NulError),

    /// UTF-8 conversion error.
    #[error("UTF-8 error")]
    Utf8(#[from] Utf8Error),

    /// An invalid pointer was encountered (can be null).
    #[error("Invalid pointer")]
    InvalidPtr,

    /// A resource is unavailable at this time.
    #[error("Resource unavailable")]
    ResourceUnavailable,

    /// IO Error.
    #[error("IO Error")]
    IoError(#[from] io::Error),
}
