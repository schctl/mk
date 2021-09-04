//! Error types.

use mk_common::FfiError;
use thiserror::Error as ThisError;

#[cfg(feature = "pam")]
use mk_pam::{PamError, RawError};

/// All error types that we handle.
#[derive(ThisError, Debug)]
pub enum MkError {
    /// PAM error.
    #[error("PAM error")]
    #[cfg(feature = "pam")]
    Pam(RawError),

    /// FFI error.
    #[error("FFI error")]
    Ffi(FfiError),

    /// Error authenticating a user
    #[error("Error authenticating a user")]
    Auth,
}

#[cfg(feature = "pam")]
impl<T> From<T> for MkError
where
    T: Into<PamError>,
{
    fn from(e: T) -> Self {
        let e: PamError = e.into();
        match e {
            PamError::Ffi(f) => Self::Ffi(f),
            PamError::Raw(f) => Self::Pam(f),
        }
    }
}

#[cfg(not(feature = "pam"))]
impl<T> From<T> for MkError
where
    T: Into<FfiError>,
{
    fn from(e: T) -> Self {
        Self::Ffi(e.into())
    }
}
