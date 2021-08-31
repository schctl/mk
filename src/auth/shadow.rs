//! User authentication using `/etc/shadow`.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use mk_pwd::Uid;

use super::Authenticator;

use crate::prelude::*;

/// Holds all the information required for authentication using `/etc/shadow`.
pub struct ShadowAuthenticator {
    /// List of all authenticated users and when they were authenticated.
    users: HashMap<Uid, Instant>,
}

impl ShadowAuthenticator {
#[must_use]
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
}

impl Authenticator for ShadowAuthenticator {
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

        if password == "x" {
            // On most systems, this is set to 'x' and the actual password is stored in `/etc/shadow/`.
            password = match shadow::Shadow::from_name(&user.name[..]) {
                Some(s) => s.password,
                None => return Ok(()),
            }
        } else if password == "*" {
            // Prevent login.
            return Err(MkError::Auth);
        }

        let auth_password =
            rpassword::prompt_password_stdout(&format!("[mk] password for {} > ", user.name)[..])
                .unwrap();

        if password != mk_crypt::crypt(&auth_password, &password)? {
            return Err(MkError::Auth);
        }

        self.users.insert(user.uid, Instant::now());
        Ok(())
    }
}
