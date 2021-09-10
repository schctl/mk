//! User authentication using PAM.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use mk_pam as pam;
use mk_pwd::Uid;

use super::Authenticator;
use crate::prelude::*;

/// Prompt a string.
fn pam_prompt(msg: &str) -> Result<pam::Response, pam::RawError> {
    Ok(pam::Response {
        resp: {
            if msg.to_lowercase().contains("password") {
                match password_from_tty!("[{}] {}", SERVICE_NAME, msg) {
                    Ok(p) => p,
                    Err(_) => return Err(pam::RawError::Conversation),
                }
            } else {
                match prompt_from_tty!("[{}] {}", SERVICE_NAME, msg) {
                    Ok(s) => s,
                    Err(_) => return Err(pam::RawError::Conversation),
                }
            }
        },
        retcode: 0,
    })
}

/// Exported PAM conversation function.
fn pam_conversation(messages: &mut [pam::conv::MessageContainer]) -> Result<(), pam::RawError> {
    for msg in messages {
        let resp = match msg.get().kind() {
            pam::MessageType::PromptEcho | pam::MessageType::Prompt => {
                Some(pam_prompt(&msg.get().get()[..])?)
            }
            pam::MessageType::ShowText => {
                println!("[{}] {}", SERVICE_NAME, msg.get().get());
                None
            }
            pam::MessageType::ShowError => {
                eprintln!("[{}] {}", SERVICE_NAME, msg.get().get());
                None
            }
            _ => None,
        };

        msg.set_response(resp);
    }

    Ok(())
}

/// PAM authentication structure. Holds all data required to begin a session with PAM.
pub struct PamAuthenticator {
    /// List of all authenticated users and when they were authenticated.
    users: HashMap<Uid, Instant>,
}

impl Default for PamAuthenticator {
    fn default() -> Self {
        Self::new()
    }
}

impl PamAuthenticator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    /// Start a PAM context and return its corresponding handle.
    fn create_context(&mut self, user: &mk_pwd::Passwd) -> MkResult<pam::Handle> {
        // Check if user is in the list of authenticated users.
        if let Some(u) = self.users.get(&user.uid) {
            if Instant::now() - *u > Duration::from_secs(600) {
                self.users.remove(&user.uid);
            }
        }

        // Create conversation function
        let conv = pam::conv::Conversation {
            conv: Box::new(pam_conversation),
        };

        let handle = pam::Handle::start(SERVICE_NAME, &user.name[..], conv)?;

        // Set requesting user.
        if let Err(e) = handle.set_item(pam::Item::RequestUser(user.name.clone())) {
            return Err(e.into());
        }

        // Authenticate user.
        if let Err(e) = handle.authenticate(None) {
            return Err(e.into());
        }

        // Get new token if required.
        if let Err(pam::PamError::Raw(pam::RawError::NewAuthTokenRequired)) = handle.validate(None)
        {
            if let Err(e) = handle.change_auth_token(None) {
                return Err(e.into());
            }
        }

        Ok(handle)
    }
}

impl Authenticator for PamAuthenticator {
    fn session<'a>(
        &mut self,
        user: &mk_pwd::Passwd,
        session: Box<dyn FnOnce() -> MkResult<()> + 'a>,
    ) -> MkResult<()> {
        let handle = self.create_context(user)?;

        handle.open_session(None)?;
        session()?;
        handle.close_session(None)?;

        Ok(())
    }
}
