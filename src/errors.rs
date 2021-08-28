//! Error types.

use std::ffi::NulError;
use std::str::Utf8Error;

use thiserror::Error;

/// All error types that we handle.
#[derive(Error, Debug)]
pub enum MkError {
    /// An interior nul byte was found.
    #[error("An interior nul byte was found")]
    NulError(#[from] NulError),
    /// Error interpreting byte sequence as utf-8.
    #[error("Error interpreting byte sequence as utf-8")]
    Utf8Error(#[from] Utf8Error),
    /// IO Error.
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    /// Error authenticating a user
    #[error("Error authenticating a user")]
    AuthError,
    /// A null pointer was found.
    #[error("A null pointer was found")]
    NullPtr,
}

impl From<libcrypt_sys::StrError> for MkError {
    fn from(e: libcrypt_sys::StrError) -> Self {
        match e {
            libcrypt_sys::StrError::NulError(e) => Self::NulError(e),
            libcrypt_sys::StrError::Utf8Error(e) => Self::Utf8Error(e),
        }
    }
}
