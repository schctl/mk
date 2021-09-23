//! Error types.

use std::ffi::NulError;
use std::io;
use std::str::Utf8Error;

pub type Result<T> = core::result::Result<T, Error>;

/// All error types that we handle.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// PAM error.
    #[error("{0}")]
    #[cfg(feature = "pam")]
    Pam(#[from] mk_pam::PamError),

    /// IO error.
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

impl From<nix::Error> for Error {
    fn from(e: nix::Error) -> Self {
        Self::Io(e.into())
    }
}

impl From<Utf8Error> for Error {
    fn from(e: Utf8Error) -> Self {
        Self::Io(io::Error::new(io::ErrorKind::Other, e))
    }
}

impl From<NulError> for Error {
    fn from(e: NulError) -> Self {
        Self::Io(io::Error::new(io::ErrorKind::Other, e))
    }
}
