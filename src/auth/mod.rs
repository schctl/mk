//! User authentication agents.

use std::io;

pub mod config;
#[cfg(feature = "pam")]
pub mod pam;
pub mod pwd;

use crate::prelude::*;

/// A user authentication agent.
pub trait UserAuthenticator {
    /// Get the user this authenticator is associated with.
    fn get_user(&self) -> &mk_pwd::Passwd;

    /// Authenticate the user and check if the user's account is valid.
    ///
    /// # Errors
    ///
    /// This function fails if the user could not be validated.
    fn validate(&self) -> Result<()>;

    /// Run a function in an authenticated session.
    ///
    /// This doesn't assume anything about the validity of the user's account.
    ///
    /// # Returns
    ///
    /// If successful, the function returns an [`Ok`] containing the result of the function.
    ///
    /// # Errors
    ///
    /// This function fails if the underlying service was unable to start or close a session.
    fn session<'a>(&self, session: Box<dyn FnOnce() -> Result<()> + 'a>) -> Result<Result<()>>;
}

/// Create a new authenticator from the given configuration.
///
/// This returns an [`std::io::Error`] of kind [`std::io::ErrorKind::NotFound`] if the feature for the
/// given type of authenticator has not been specified.
#[allow(unreachable_patterns)]
pub fn new(user: mk_pwd::Passwd, cfg: config::AuthConfig) -> Result<Box<dyn UserAuthenticator>> {
    Ok(match cfg.ty {
        #[cfg(feature = "pam")]
        config::AuthService::Pam => Box::new(pam::PamAuthenticator::new(user, cfg.rules)?),
        config::AuthService::Pwd => Box::new(pwd::PwdAuthenticator::new(user, cfg.rules)?),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("unsupported authenticator {:?}", cfg.ty),
            )
            .into())
        }
    })
}
