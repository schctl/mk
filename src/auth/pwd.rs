//! User authentication using the system password database.
//!
//! This is the fallback authenticator type, and is available on all platforms.

use std::io::{Error, ErrorKind};

use nix::unistd::User;

use super::{Rules, UserAuthenticator};
use crate::prelude::*;

/// Holds all the information required for authentication using the system password database.
pub struct PwdAuthenticator {
    user: User,
    #[allow(unused)]
    rules: Rules,
}

impl PwdAuthenticator {
    pub fn new(user: User, rules: Rules) -> Result<Self> {
        // Result only for consistency
        Ok(Self { user, rules })
    }

    /// Authenticate the user's account.
    fn authenticate(&self) -> Result<()> {
        // Authenticate if user doesn't have a password.
        #[allow(unused_mut)]
        let mut password = match self.user.passwd.to_str() {
            Ok(e) => e.to_owned(),
            Err(_) => {
                return Err(Error::new(ErrorKind::Other, "non utf-8 passwords unsupported").into())
            }
        };

        #[cfg(feature = "shadow")]
        match &password[..] {
            "*" => return Err(Error::new(ErrorKind::PermissionDenied, "disallowed login").into()),
            // > On some systems, this field is set to x, and the user password is stored in
            // > the /etc/shadow file.
            "x" => {
                let spwd = match mk_shadow::Spwd::from_name(&self.user.name[..])?
                    .password
                    .to_str()
                {
                    Ok(e) => e.to_owned(),
                    Err(_) => {
                        return Err(
                            Error::new(ErrorKind::Other, "non utf-8 passwords unsupported").into(),
                        )
                    }
                };

                if let "*" | "!" = &spwd[..] {
                    return Err(Error::new(ErrorKind::PermissionDenied, "disallowed login").into());
                }

                password = spwd;
            }
            _ => {}
        };

        #[cfg(not(feature = "shadow"))]
        match &password[..] {
            "*" | "x" => {
                return Err(Error::new(ErrorKind::PermissionDenied, "disallowed login").into())
            }
            _ => {}
        };

        if !pwhash::unix::verify(
            &password_from_tty!("[{}] Password: ", SERVICE_NAME)?,
            &password[..],
        ) {
            return Err(Error::new(ErrorKind::PermissionDenied, "permission denied").into());
        }

        Ok(())
    }
}

impl UserAuthenticator for PwdAuthenticator {
    fn get_user(&self) -> &User {
        &self.user
    }

    fn validate(&mut self) -> Result<()> {
        self.authenticate()
    }

    fn session<'a>(
        &mut self,
        session: Box<dyn FnOnce() -> Result<()> + 'a>,
        _: &User,
    ) -> Result<Result<()>> {
        Ok(session())
    }
}
