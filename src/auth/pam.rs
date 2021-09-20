//! User authentication using PAM.

use mk_pam as pam;

use super::{Rules, UserAuthenticator};
use crate::prelude::*;

/// Prompt a string.
fn pam_prompt(msg: &str) -> core::result::Result<pam::Response, pam::PamError> {
    Ok(pam::Response {
        resp: {
            if msg.to_lowercase().contains("password") {
                match password_from_tty!("[{}] {}", SERVICE_NAME, msg) {
                    Ok(p) => p,
                    Err(_) => return Err(pam::PamError::Conversation),
                }
            } else {
                match prompt_from_tty!("[{}] {}", SERVICE_NAME, msg) {
                    Ok(s) => s,
                    Err(_) => return Err(pam::PamError::Conversation),
                }
            }
        },
        retcode: 0,
    })
}

/// Exported PAM conversation function.
fn pam_conversation(
    messages: &mut [pam::conv::MessageContainer],
) -> core::result::Result<(), pam::PamError> {
    for msg in messages {
        let resp = match msg.get().kind() {
            pam::MessageType::PromptEcho | pam::MessageType::Prompt => {
                Some(pam_prompt(&msg.get().contents()[..])?)
            }
            pam::MessageType::ShowText => {
                println!("[{}] {}", SERVICE_NAME, msg.get().contents());
                None
            }
            pam::MessageType::ShowError => {
                eprintln!("[{}] {}", SERVICE_NAME, msg.get().contents());
                None
            }
            _ => None,
        };

        msg.resp = resp;
    }

    Ok(())
}

/// PAM authentication structure. Holds all data required to begin a session with PAM.
pub struct PamAuthenticator {
    user: mk_pwd::Passwd,
    handle: pam::Handle,
    #[allow(unused)]
    rules: Rules,
}

impl PamAuthenticator {
    pub fn new(user: mk_pwd::Passwd, rules: Rules) -> Result<Self> {
        let handle = pam::Handle::start(SERVICE_NAME, &user.name[..], Box::new(pam_conversation))?;

        handle.items().set_request_user(&user.name[..])?;

        // TODO: host name

        Ok(Self {
            user,
            handle,
            rules,
        })
    }
}

impl UserAuthenticator for PamAuthenticator {
    fn get_user(&self) -> &mk_pwd::Passwd {
        &self.user
    }

    fn validate(&self) -> Result<()> {
        self.handle.authenticate(None)?;
        self.handle.validate(None)?;
        Ok(())
    }

    fn session<'a>(&self, session: Box<dyn FnOnce() -> Result<()> + 'a>) -> Result<Result<()>> {
        self.handle.open_session(None)?;
        let res = session();
        self.handle.close_session(None)?;
        Ok(res)
    }
}
