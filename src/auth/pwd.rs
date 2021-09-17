//! User authentication using the system password database.
//!
//! This is the fallback authenticator type, and is available on all platforms.

use std::io;

use mk_pwd::Passwd;

use super::{Rules, UserAuthenticator};
use crate::prelude::*;

/// Holds all the information required for authentication using the system password database.
pub struct PwdAuthenticator {
    user: Passwd,
    #[allow(unused)]
    rules: Rules,
}

impl PwdAuthenticator {
    pub fn new(user: Passwd, rules: Rules) -> Result<Self> {
        // Result only for consistency
        Ok(Self { user, rules })
    }

    /// Authenticate the user's account.
    fn authenticate(&self) -> Result<()> {
        // Authenticate if user doesn't have a password.
        let password = match self.user.password.clone() {
            Some(p) => p,
            None => return Ok(()),
        };

        #[cfg(feature = "shadow")]
        let password = match &password[..] {
            "*" => {
                return Err(
                    io::Error::new(io::ErrorKind::PermissionDenied, "disallowed login").into(),
                )
            }
            // > On most modern systems, this field is set to x, and the user password is stored in
            // > the /etc/shadow file.
            "x" => {
                let spwd = mk_shadow::Spwd::from_name(&self.user.name[..])?;

                if let "*" | "!" = &spwd.password[..] {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "disallowed login",
                    )
                    .into());
                }

                spwd.password
            }
            _ => password,
        };

        #[cfg(not(feature = "shadow"))]
        let password = match &password[..] {
            "*" | "x" => {
                return Err(
                    io::Error::new(io::ErrorKind::PermissionDenied, "disallowed login").into(),
                )
            }
            _ => password,
        };

        if !pwhash::unix::verify(
            &password_from_tty!("[{}] Password: ", SERVICE_NAME)?,
            &password[..],
        ) {
            return Err(
                io::Error::new(io::ErrorKind::PermissionDenied, "permission denied").into(),
            );
        }

        Ok(())
    }
}

impl UserAuthenticator for PwdAuthenticator {
    fn get_user(&self) -> &Passwd {
        &self.user
    }

    fn validate(&self) -> Result<()> {
        self.authenticate()
    }

    fn session<'a>(&self, session: Box<dyn FnOnce() -> Result<()> + 'a>) -> Result<Result<()>> {
        Ok(session())
    }
}
