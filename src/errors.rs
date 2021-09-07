//! Error types.

use std::io;
use thiserror::Error as ThisError;

#[cfg(feature = "pam")]
use mk_pam::{PamError, RawError};

/// All error types that we handle.
#[derive(ThisError, Debug)]
pub enum MkError {
    /// PAM error.
    #[error("{0}")]
    #[cfg(feature = "pam")]
    Pam(RawError),

    /// IO Error.
    #[error("{0}")]
    Io(#[from] io::Error),
}

#[cfg(feature = "pam")]
impl From<PamError> for MkError {
    fn from(e: PamError) -> Self {
        match e {
            PamError::Raw(r) => Self::Pam(r),
            PamError::Io(r) => Self::Io(r),
        }
    }
}
