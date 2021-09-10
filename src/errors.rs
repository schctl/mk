//! Error types.

use std::io;
use thiserror::Error as ThisError;

/// All error types that we handle.
#[derive(ThisError, Debug)]
pub enum MkError {
    /// PAM error.
    #[error("{0}")]
    #[cfg(feature = "pam")]
    Pam(#[from] mk_pam::RawError),

    /// IO Error.
    #[error("{0}")]
    Io(#[from] io::Error),
}

#[cfg(feature = "pam")]
impl From<mk_pam::PamError> for MkError {
    fn from(e: mk_pam::PamError) -> Self {
        match e {
            mk_pam::PamError::Raw(r) => Self::Pam(r),
            mk_pam::PamError::Io(r) => Self::Io(r),
        }
    }
}
