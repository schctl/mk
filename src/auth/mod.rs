//! User authentication agents.

use std::io;
use std::time;

#[cfg(feature = "pam")]
pub mod pam;
pub mod pwd;

use crate::prelude::*;

/// Provides methods to authenticate a user.
///
/// Additional required information must be held by the implementer. The intention is for an
/// Authenticator to be dumped to a file and recovered between sessions.
pub trait Authenticator: Send + Sync {
    /// Run a function in an authenticated session.
    fn session<'a>(
        &mut self,
        user: &mk_pwd::Passwd,
        session: Box<dyn FnOnce() -> MkResult<()> + 'a>,
    ) -> MkResult<()>;

    /// Set authentication timeout. This will fail any authentication attempts after waiting
    /// for the provided duration.
    fn set_timeout(&mut self, _: Option<time::Duration>) -> MkResult<()> {
        Ok(())
    }

    /// Get the authentication timeout.
    fn get_timeout(&self) -> &Option<time::Duration> {
        &None
    }
}

/// All supported authenticator types.
#[allow(unused)]
#[non_exhaustive]
#[derive(Debug)]
pub enum Supported {
    /// [`pam::PamAuthenticator`] authentication.
    #[cfg(feature = "pam")]
    Pam,
    /// [`pwd::PwdAuthenticator`] authentication.
    Pwd,
}

/// Create a new authenticator from the supported types.
///
/// This returns an [`std::io::Error`] of kind [`std::io::ErrorKind::NotFound`] if the feature for the given type of authenticator
/// has not been specified.
#[allow(unreachable_patterns)]
pub fn new(_type: &Supported) -> MkResult<Box<dyn Authenticator>> {
    match _type {
        #[cfg(feature = "pam")]
        Supported::Pam => Ok(Box::new(pam::PamAuthenticator::new())),
        Supported::Pwd => Ok(Box::new(pwd::PwdAuthenticator::new())),
        _ => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("unsupported authenticator {:?}", _type),
        )
        .into()),
    }
}
