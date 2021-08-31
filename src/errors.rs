//! Error types.

use mk_common::errors::FfiError;
use mk_pam::errors::PamError;
use thiserror::Error as ThisError;

/// All error types that we handle.
#[derive(ThisError, Debug)]
pub enum MkError {
    /// PAM error.
    #[error("PAM error")]
    Pam(PamError),

    /// FFI error.
    #[error("FFI error")]
    Ffi(FfiError),

    /// Error authenticating a user
    #[error("Error authenticating a user")]
    Auth,
}

impl<T> From<T> for MkError
where
    T: Into<PamError>,
{
    fn from(e: T) -> Self {
        let e: PamError = e.into();
        match e {
            PamError::Ffi(f) => Self::Ffi(f),
            _ => Self::Pam(e),
        }
    }
}
