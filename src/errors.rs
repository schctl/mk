//! Error types.

use std::ffi::NulError;
use std::str::Utf8Error;

use nix::errno::Errno;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MkError {
    /// An interior nul byte was found.
    #[error("An interior nul byte was found")]
    NulError(#[from] NulError),
    /// Error interpreting byte sequence as utf-8.
    #[error("Error interpreting byte sequence as utf-8")]
    Utf8Error(#[from] Utf8Error),
    /// Unable to authenticate a user.
    #[error("Error authenticating a user")]
    AuthError,
    /// System call error number.
    #[error("Error number from a system call [errno: {0}]")]
    Errno(#[from] Errno),
}

impl From<libcrypt_sys::StrError> for MkError {
    fn from(e: libcrypt_sys::StrError) -> Self {
        match e {
            libcrypt_sys::StrError::NulError(e) => Self::NulError(e),
            libcrypt_sys::StrError::Utf8Error(e) => Self::Utf8Error(e),
        }
    }
}
