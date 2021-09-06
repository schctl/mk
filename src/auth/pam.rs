//! User authentication using [`PAM`].
//!
//! [`PAM`]: https://en.wikipedia.org/wiki/Pluggable_authentication_module

use std::collections::HashMap;
use std::time::{Duration, Instant};

use mk_pam as pam;
use mk_pwd::Uid;

use super::Authenticator;
use crate::prelude::*;

/// PAM authentication structure. Holds all data required to begin a session with PAM.
pub struct PamAuthenticator {
    /// List of all authenticated users and when they were authenticated.
    users: HashMap<Uid, Instant>,
}

impl PamAuthenticator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
}

impl Authenticator for PamAuthenticator {
    fn authenticate(&mut self, user: &mk_pwd::Passwd) -> MkResult<()> {
        // Check if user is in the list of authenticated users.
        if let Some(u) = self.users.get(&user.uid) {
            if Instant::now() - *u < Duration::from_secs(600) {
                return Ok(());
            } else {
                self.users.remove(&user.uid);
            }
        }

        let username_str = user.name.clone();

        // Create conversation function
        let conv = pam::conv::Conversation {
            conv: Box::new(move |msg| match msg {
                pam::conv::Message::PromptEcho(m) | pam::conv::Message::Prompt(m) => {
                    Some(pam::conv::Response {
                        resp: match prompt!(
                            m.to_lowercase().contains("password"),
                            "[{}] {}",
                            username_str,
                            m
                        ) {
                            Ok(p) => p,
                            Err(_) => return None,
                        },
                        retcode: 0,
                    })
                }
                pam::conv::Message::ShowText(m) => {
                    println!("[{}] {}", SERVICE_NAME, m);
                    None
                }
                pam::conv::Message::ShowError(m) => {
                    eprintln!("[{}] {}", SERVICE_NAME, m);
                    None
                }
            }),
        };

        let handle = pam::Handle::start(SERVICE_NAME, &user.name[..], conv)?;

        // Set requesting user.
        if let Err(e) = handle.set_item(pam::Item::RequestUser(user.name.clone())) {
            handle.end()?;
            return Err(e.into());
        }

        // Authenticate user.
        if let Err(e) = handle.authenticate(None) {
            handle.end()?;
            return Err(e.into());
        }

        // Get new token if required.
        if let Err(pam::PamError::Raw(pam::RawError::NewAuthTokenRequired)) = handle.validate(None)
        {
            if let Err(e) = handle.change_auth_token(None) {
                handle.end()?;
                return Err(e.into());
            }
        }

        handle.end()?;
        Ok(())
    }
}
