//! User authentication using `/etc/shadow`.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use nix::unistd::Uid;

use super::Authenticator;

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
    fn authenticate(&mut self, user: Uid) -> Result<(), ()> {
        // Original user id.
        let uid = user;

        // First, get the user entry in `/etc/passwd/`.
        let user = match pwd::Passwd::from_uid(user.as_raw()) {
            Some(u) => u,
            None => return Err(()),
        };

        // Check if user is in the list of authenticated users.
        if let Some(u) = self.users.get(&uid) {
            if Instant::now() - *u < Duration::from_secs(600) {
                return Ok(());
            } else {
                self.users.remove(&uid);
            }
        }

        // Authenticate if user doesn't have a password.
        let mut password = match user.passwd {
            Some(p) => p,
            None => return Ok(()),
        };

        // On most systems, this is set to 'x' and the actual password is stored in `/etc/shadow/`.
        if password == "x" {
            password = match shadow::Shadow::from_name(&user.name[..]) {
                Some(s) => s.password,
                None => return Err(()),
            }
        } else if password == "*" {
            return Err(());
        }

        // Prompt caller for password
        let auth_password = rpassword::prompt_password_stdout("Password > ").unwrap();

        // Check if provided password matches the same as the one in the entry.
        if password != libcrypt_sys::crypt(&auth_password, &password) {
            return Err(());
        }

        // Add user to list of authenticated users.
        self.users.insert(uid, Instant::now());
        Ok(())
    }
}
