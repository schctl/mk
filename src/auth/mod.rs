//! User authentication agents.

pub mod pam;
pub mod shadow;

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

pub enum Supported {
    /// [`pam::PamAuthenticator`] authentication.
    Pam,
    /// [`shadow::ShadowAuthenticator`] authentication.
    Shadow,
}

/// Create a new authenticator from the supported types.
pub fn new_authenticator(_type: Supported) -> MkResult<Box<dyn Authenticator>> {
    Ok(match _type {
        Supported::Pam => Box::new(pam::PamAuthenticator::new()),
        Supported::Shadow => Box::new(shadow::ShadowAuthenticator::new()),
    })
}
