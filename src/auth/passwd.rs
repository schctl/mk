//! User authentication using `/etc/passwd`.
//!
//! This is the fallback authenticator type, and is available on all platforms.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use mk_pwd::Uid;

use super::Authenticator;
use crate::prelude::*;

/// Holds all the information required for authentication using `/etc/passwd`.
pub struct PasswdAuthenticator {
    /// List of all authenticated users and when they were authenticated.
    users: HashMap<Uid, Instant>,
}

impl PasswdAuthenticator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
}

impl Authenticator for PasswdAuthenticator {
    fn authenticate(&mut self, user: &mk_pwd::Passwd) -> MkResult<()> {
        // Check if user is in the list of authenticated users.
        if let Some(u) = self.users.get(&user.uid) {
            if Instant::now() - *u < Duration::from_secs(600) {
                return Ok(());
            } else {
                self.users.remove(&user.uid);
            }
        }

        // Authenticate if user doesn't have a password.
        let mut password = match user.password.clone() {
            Some(p) => p,
            None => return Ok(()),
        };

        let password = match &password[..] {
            "*" => return Err(MkError::Auth),
            // > On most modern systems, this field is set to x, and the user password is stored in
            // > the /etc/shadow file.
            #[cfg(feature = "sdw")]
            "x" => match shadow::Shadow::from_name(&user.name[..]) {
                Some(s) => match &s.password[..] {
                    "*" | "!" => return Err(MkError::Auth),
                    _ => s.password,
                },
                None => return Ok(()),
            },
            _ => password,
        };

        let auth_password = prompt!(true, "[{}] Password: ", user.name)?;

        if password != mk_crypt::crypt(&auth_password, &password)? {
            return Err(MkError::Auth);
        }

        self.users.insert(user.uid, Instant::now());
        Ok(())
    }
}
