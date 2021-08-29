//! User authenticator using PAM.

use std::ffi::CString;
use std::os::raw::{c_int, c_void};

use mk_pam as pam;
use mk_pam::ffi as pamffi;

use super::Authenticator;

use crate::prelude::*;
use crate::prompt;

/// PAM authentication structure. Holds all data required to begin a session with PAM.
pub struct PamAuthenticator {}

impl PamAuthenticator {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Authenticator for PamAuthenticator {
    fn authenticate(&mut self, user: &mk_pwd::Passwd) -> MkResult<()> {
        let username_str = String::from(&user.name[..]);
        let username = CString::new(&user.name[..])?;

        // Create conversation function
        let conv = pam::conv::Conversation {
            conv: Box::new(move |msg| match msg {
                pam::conv::Message::PromptEcho(m) | pam::conv::Message::Prompt(m) => {
                    Some(pam::conv::Response {
                        resp: match prompt::prompt(
                            &username_str[..],
                            &m[..],
                            m.to_lowercase().contains("password"),
                            true,
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

        let handle = pam::Handle::start("mk", &user.name[..], conv).unwrap();

        // Set items
        let ret = unsafe {
            pamffi::pam_set_item(
                handle.interior,
                pamffi::PAM_RUSER as c_int,
                username.as_ptr() as *const c_void,
            )
        };

        if ret != pamffi::PAM_SUCCESS as c_int {
            println!("Failed to set PAM user {}", ret);
            return Err(MkError::AuthError);
        }

        // Authenticate user
        let ret = unsafe { pamffi::pam_authenticate(handle.interior, 0) };

        if ret != pamffi::PAM_SUCCESS as c_int {
            println!("Failed to authenticate user {}", ret);
            unsafe { pamffi::pam_end(handle.interior, ret) };
            return Err(MkError::AuthError);
        }

        // Check if the user's account is still active, and has permission to access the system
        // at this time.
        let ret = unsafe { pamffi::pam_acct_mgmt(handle.interior, 0) };

        if ret == pamffi::PAM_NEW_AUTHTOK_REQD as c_int {
            let ret = unsafe { pamffi::pam_chauthtok(handle.interior, 0) };
            if ret != pamffi::PAM_SUCCESS as c_int {
                println!("Failed to authenticate user {}", ret);
                unsafe { pamffi::pam_end(handle.interior, ret) };
                return Err(MkError::AuthError);
            }
        }

        Ok(())
    }
}
