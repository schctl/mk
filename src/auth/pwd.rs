//! User authentication using the system password database.
//!
//! This is the fallback authenticator type, and is available on all platforms.

use std::collections::HashMap;
use std::io;
use std::time::{Duration, Instant};

use mk_pwd::Uid;

use super::Authenticator;
use crate::prelude::*;

/// Holds all the information required for authentication using the system password database.
pub struct PwdAuthenticator {
    /// List of all authenticated users and when they were authenticated.
    users: HashMap<Uid, Instant>,
}

impl Default for PwdAuthenticator {
    fn default() -> Self {
        Self::new()
    }
}

impl PwdAuthenticator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    fn authenticate(&mut self, user: &mk_pwd::Passwd) -> MkResult<()> {
        // Check if user is in the list of authenticated users.
        if let Some(u) = self.users.get(&user.uid) {
            if Instant::now() - *u < Duration::from_secs(600) {
                return Ok(());
            }

            self.users.remove(&user.uid);
        }

        // Authenticate if user doesn't have a password.
        let password = match user.password.clone() {
            Some(p) => p,
            None => return Ok(()),
        };

        #[cfg(feature = "shadow")]
        let password = match &password[..] {
            // Not sure how to handle this
            "*" => {
                return Err(
                    io::Error::new(io::ErrorKind::PermissionDenied, "disallowed login").into(),
                )
            }
            // > On most modern systems, this field is set to x, and the user password is stored in
            // > the /etc/shadow file.
            "x" => {
                let spwd = mk_shadow::Spwd::from_name(&user.name[..])?;

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

        self.users.insert(user.uid, Instant::now());
        Ok(())
    }
}

impl Authenticator for PwdAuthenticator {
    fn session<'a>(
        &mut self,
        user: &mk_pwd::Passwd,
        session: Box<dyn FnOnce() -> MkResult<()> + 'a>,
    ) -> MkResult<()> {
        self.authenticate(user)?;
        session()
    }
}
