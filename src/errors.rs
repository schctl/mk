//! Error types.

use std::io;

pub type Result<T> = core::result::Result<T, Error>;

/// All error types that we handle.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// PAM error.
    #[error("{0}")]
    #[cfg(feature = "pam")]
    Pam(#[from] mk_pam::PamError),

    /// IO Error.
    #[error("{0}")]
    Io(#[from] io::Error),
}

#[cfg(feature = "pam")]
impl From<mk_pam::Error> for Error {
    fn from(e: mk_pam::Error) -> Self {
        match e {
            mk_pam::Error::Raw(r) => Self::Pam(r),
            mk_pam::Error::Io(r) => Self::Io(r),
        }
    }
}
