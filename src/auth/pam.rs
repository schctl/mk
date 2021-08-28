//! User authenticator using PAM.

use std::ffi::{CStr, CString};
use std::os::raw::{c_int, c_void};

use mk_pam::ffi as pam;

use super::Authenticator;

use crate::prelude::*;

/// PAM conversation function.
///
/// From the Linux man pages.
/// > The PAM library uses an application-defined callback to allow a direct communication between
/// > a loaded module and the application. This callback is specified by the struct `pam_conv`
/// > passed to `pam_start(3)` at the start of the transaction.
///
/// # Arguments
///
/// * `num_msgs` - The number of message pointers held in the `msgs` argument.
/// * `msgs` - Array of [`PamMessage`] pointers.
/// * `response` - Array of [`PamResponse`] pointers.
/// * `_appdata_ptr` - Set to the second element of the structure this function was provided in.
extern "C" fn pam_conversation(
    num_msgs: c_int,
    msgs: *mut *const pam::pam_message,
    response: *mut *mut pam::pam_response,
    _appdata_ptr: *mut c_void,
) -> c_int {
    // Everything in this is pretty unsafe.
    // I don't even feel like writing a safety block.
    unsafe {
        for i in 0..num_msgs as isize {
            let msg = *msgs.offset(i);
            let response = *response.offset(i);

            match (*msg).msg_style as u32 {
                pam::PAM_PROMPT_ECHO_OFF | pam::PAM_PROMPT_ECHO_ON => {
                    // Create a response for this message.
                    *response = pam::pam_response {
                        // Read password from terminal and get a mut ptr to it.
                        resp: CString::new(
                            rpassword::read_password_from_tty(Some("[mk] Password > ")).unwrap(),
                        )
                        .unwrap()
                        .into_raw(),
                        // Currently unused and 0 is expected.
                        resp_retcode: 0,
                    };
                }
                pam::PAM_TEXT_INFO => {
                    println!("{}", CStr::from_ptr((*msg).msg).to_str().unwrap());
                }
                pam::PAM_ERROR_MSG => {
                    eprintln!("{}", CStr::from_ptr((*msg).msg).to_str().unwrap());
                }
                _ => {}
            }
        }
    }

    0
}

/// Linux PAM authentication structure. Holds all data required to begin a session with PAM.
pub struct PamAuthenticator {}

impl PamAuthenticator {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Authenticator for PamAuthenticator {
    fn authenticate(&mut self, user: &mk_pwd::Passwd) -> MkResult<()> {
        // See http://uw714doc.sco.com/en/SEC_pam/pam_appl-3.html

        let username = CString::new(&user.name[..])?;
        let service = CString::new("mk")?;

        let mut pamh: *mut pam::pam_handle = std::ptr::null_mut();

        // Create the pam conversation structure, holding a callback to our pam conversation function.
        let pam_conv = pam::pam_conv {
            conv: Some(pam_conversation),
            appdata_ptr: std::ptr::null_mut(),
        };

        // Start the pam conversation.
        let ret =
            unsafe { pam::pam_start(service.as_ptr(), username.as_ptr(), &pam_conv, &mut pamh) };

        if ret != pam::PAM_SUCCESS as c_int {
            println!("Failed to started PAM {}", ret);
            return Err(MkError::AuthError);
        }

        // Set items
        let ret = unsafe {
            pam::pam_set_item(
                pamh,
                pam::PAM_RUSER as c_int,
                username.as_ptr() as *const c_void,
            )
        };

        if ret != pam::PAM_SUCCESS as c_int {
            println!("Failed to set PAM user {}", ret);
            return Err(MkError::AuthError);
        }

        // Authenticate user
        let ret = unsafe { pam::pam_authenticate(pamh, 0) };

        if ret != pam::PAM_SUCCESS as c_int {
            println!("Failed to authenticate user {}", ret);
            unsafe { pam::pam_end(pamh, ret) };
            return Err(MkError::AuthError);
        }

        // Check if the user's account is still active, and has permission to access the system
        // at this time.
        let ret = unsafe { pam::pam_acct_mgmt(pamh, 0) };

        if ret == pam::PAM_NEW_AUTHTOK_REQD as c_int {
            let ret = unsafe { pam::pam_chauthtok(pamh, 0) };
            if ret != pam::PAM_SUCCESS as c_int {
                println!("Failed to authenticate user {}", ret);
                unsafe { pam::pam_end(pamh, ret) };
                return Err(MkError::AuthError);
            }
        }

        Ok(())
    }
}
