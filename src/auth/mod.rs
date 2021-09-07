//! User authentication agents.

#[cfg(feature = "pam")]
pub mod pam;
pub mod pwd;

use mk_common::*;

use crate::prelude::*;

/// Provides methods to authenticate a user.
///
/// Additional required information must be held by the implementer. The intention is for an
/// Authenticator to be dumped to a file and recovered between sessions.
pub trait Authenticator {
    /// Run a function in an authenticated session.
    fn session<'a>(
        &mut self,
        user: &mk_pwd::Passwd,
        session: Box<dyn FnOnce() -> MkResult<()> + 'a>,
    ) -> MkResult<()>;
}

/// All supported authenticator types.
///
/// Not checked for whether they are enabled through a feature.
#[allow(unused)]
#[non_exhaustive]
pub enum Supported {
    /// [`pam::PamAuthenticator`] authentication.
    Pam,
    /// [`pwd::PwdAuthenticator`] authentication.
    Pwd,
}

/// Create a new authenticator from the supported types.
///
/// This returns an [`io::Error`] of kind [`io::ErrorKind::NotFound`] if the feature for the given type of authenticator
/// has not been specified.
#[allow(unreachable_patterns)]
pub fn new(_type: Supported) -> MkResult<Box<dyn Authenticator>> {
    match _type {
        #[cfg(feature = "pam")]
        Supported::Pam => Ok(Box::new(pam::PamAuthenticator::new())),
        Supported::Pwd => Ok(Box::new(pwd::PwdAuthenticator::new())),
        _ => {
            io_err!(NotFound, "no supported authenticator")
        }
    }
}
