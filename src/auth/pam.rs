//! User authentication using PAM.

use mk_common::get_host_name;
use mk_pam as pam;
use mk_pwd::Passwd;

use super::{Rules, UserAuthenticator};
use crate::prelude::*;

/// Exported PAM conversation function.
fn pam_conversation(
    messages: &mut [pam::MessageContainer],
) -> core::result::Result<(), pam::PamError> {
    for msg in messages {
        match msg.msg.kind() {
            pam::MessageType::Prompt => {
                msg.resp = Some(pam::Response {
                    resp: {
                        match prompt_from_tty!("[{}] {}", SERVICE_NAME, &msg.msg.contents()[..]) {
                            Ok(p) => p,
                            Err(_) => return Err(pam::PamError::Conversation),
                        }
                    },
                })
            }
            pam::MessageType::PromptNoEcho => {
                msg.resp = Some(pam::Response {
                    resp: {
                        match password_from_tty!("[{}] {}", SERVICE_NAME, &msg.msg.contents()[..]) {
                            Ok(p) => p,
                            Err(_) => return Err(pam::PamError::Conversation),
                        }
                    },
                })
            }
            pam::MessageType::ShowText => {
                println!("[{}] {}", SERVICE_NAME, msg.msg.contents());
            }
            pam::MessageType::ShowError => {
                eprintln!("[{}] {}", SERVICE_NAME, msg.msg.contents());
            }
            _ => {}
        }
    }

    Ok(())
}

/// PAM authentication structure. Holds all data required to begin a session with PAM.
pub struct PamAuthenticator {
    user: Passwd,
    handle: pam::Handle,
    #[allow(unused)]
    rules: Rules,
}

impl PamAuthenticator {
    pub fn new(user: Passwd, rules: Rules) -> Result<Self> {
        let mut handle =
            pam::Handle::start(SERVICE_NAME, &user.name[..], Box::new(pam_conversation))?;

        let mut items = handle.items();
        items.set_request_user(&user.name[..])?;
        items.set_request_host(&get_host_name()?[..])?;

        Ok(Self {
            user,
            handle,
            rules,
        })
    }
}

impl UserAuthenticator for PamAuthenticator {
    fn get_user(&self) -> &Passwd {
        &self.user
    }

    fn validate(&mut self) -> Result<()> {
        self.handle.authenticate(pam::Flags::NONE)?;

        match self.handle.validate(pam::Flags::NONE) {
            Ok(_) => {}
            Err(pam::Error::Raw(pam::PamError::NewAuthTokenRequired)) => {
                self.handle
                    .change_auth_token(pam::Flags::CHANGE_EXPIRED_AUTH_TOKEN)?;
            }
            Err(e) => return Err(e.into()),
        };

        Ok(())
    }

    fn session<'a>(
        &mut self,
        session: Box<dyn FnOnce() -> Result<()> + 'a>,
        session_user: &Passwd,
    ) -> Result<Result<()>> {
        self.handle.items().set_user(&session_user.name[..])?;
        self.handle.set_creds(pam::Flags::REINITIALIZE_CREDS)?;
        self.handle.open_session(pam::Flags::NONE)?;

        let res = session();

        self.handle.close_session(pam::Flags::NONE)?;
        self.handle
            .set_creds(pam::Flags::DELETE_CREDS | pam::Flags::SILENT)?;

        Ok(res)
    }
}
