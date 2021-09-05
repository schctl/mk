//! User authentication agents.

use std::io;

pub mod pwd;

#[cfg(feature = "pam")]
pub mod pam;

use mk_common::*;

use crate::prelude::*;

/// Provides methods to authenticate a user.
///
/// Additional required information must be held by the implementer. The intention is for an
/// Authenticator to be dumped to a file and recovered between sessions.
pub trait Authenticator {
    /// Authenticate validity of a user.
    ///
    /// # Returns
    ///
    /// `()` if the user has been authenticated.
    ///
    /// # Errors
    ///
    /// [`MkError::Auth`] - if the user failed the authentication.
    /// [`MkError`] - any other error.
    fn authenticate(&mut self, user: &mk_pwd::Passwd) -> MkResult<()>;
}

/// All supported authenticator types.
///
/// Not checked for whether they are enabled through a feature.
pub enum Supported {
    /// [`pam::PamAuthenticator`] authentication.
    Pam,
    /// [`passwd::PwdAuthenticator`] authentication.
    Pwd,
}

/// Create a new authenticator from the supported types.
///
/// This returns [`FfiError::ResourceUnavailable`] if the feature for the given type of authenticator
/// has not been specified.
pub fn new(_type: Supported) -> MkResult<Box<dyn Authenticator>> {
    Ok(match _type {
        #[cfg(feature = "pam")]
        Supported::Pam => Box::new(pam::PamAuthenticator::new()),
        Supported::Pwd => Box::new(pwd::PwdAuthenticator::new()),
        _ => {
            return Err(
                io::Error::new(io::ErrorKind::NotFound, "no supported authenticator").into(),
            )
        }
    })
}
