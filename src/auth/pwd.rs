//! User authentication using the system `<pwd.h>`.
//!
//! This is the fallback authenticator type, and is available on all platforms.

use std::collections::HashMap;
use std::io;
use std::time::{Duration, Instant};

use mk_common::*;
use mk_pwd::Uid;

use super::Authenticator;
use crate::prelude::*;

/// Holds all the information required for authentication using the system `<pwd.h>`.
pub struct PwdAuthenticator {
    /// List of all authenticated users and when they were authenticated.
    users: HashMap<Uid, Instant>,
}

impl PwdAuthenticator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
}

impl Authenticator for PwdAuthenticator {
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

        #[cfg(feature = "sdw")]
        let password = match &password[..] {
            // Not sure how to handle this
            "*" => auth_bail!("disallowed login"),
            // > On most modern systems, this field is set to x, and the user password is stored in
            // > the /etc/shadow file.
            "x" => match shadow::Shadow::from_name(&user.name[..]) {
                Some(s) => match &s.password[..] {
                    "*" | "!" => auth_bail!("disallowed login"),
                    _ => s.password,
                },
                None => auth_bail!("disallowed login"),
            },
            _ => password,
        };

        #[cfg(not(feature = "sdw"))]
        let password = match &password[..] {
            "*" | "x" => auth_bail!("disallowed login"),
            _ => password,
        };

        if password != mk_crypt::crypt(&prompt!(true, "[{}] Password: ", user.name)?, &password)? {
            auth_bail!("Authentication failed");
        }

        self.users.insert(user.uid, Instant::now());
        Ok(())
    }
}
