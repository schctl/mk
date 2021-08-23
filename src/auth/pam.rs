//! User authenticator using Linux PAM.

use std::cell::UnsafeCell;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};

use pam_sys::{raw::*, types::*};

use super::Authenticator;

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
    msgs: *mut *mut PamMessage,
    response: *mut *mut PamResponse,
    _appdata_ptr: *mut c_void,
) -> c_int {
    // Everything in this is pretty unsafe.
    // I don't even feel like writing a safety block.
    unsafe {
        for i in 0..num_msgs as isize {
            let msg = *msgs.offset(i);
            let response = *response.offset(i);

            match (*msg).msg_style.into() {
                PamMessageStyle::PROMPT_ECHO_ON | PamMessageStyle::PROMPT_ECHO_OFF => {
                    // Create a response for this message
                    *response = PamResponse {
                        // Read password from terminal and get a mut ptr to it.
                        resp: rpassword::read_password_from_tty(Some("Password: "))
                            .unwrap()
                            .as_mut_str()
                            .as_mut_ptr() as *mut c_char,
                        // Currently unused and 0 is expected
                        resp_retcode: 0,
                    };
                }
                PamMessageStyle::TEXT_INFO => {
                    println!("{}", CStr::from_ptr((*msg).msg).to_str().unwrap());
                }
                PamMessageStyle::ERROR_MSG => {
                    eprintln!("{}", CStr::from_ptr((*msg).msg).to_str().unwrap());
                }
            }
        }
    }

    PamReturnCode::SUCCESS as c_int
}

/// Linux PAM authentication structure. Holds all data required to begin a session with PAM.
pub struct PamAuthenticator {}

impl Authenticator for PamAuthenticator {
    fn authenticate(&self, username: &str, password: &str) -> Result<(), ()> {
        // We need mutable pointers to these later on.
        let username = UnsafeCell::new(username);
        let _password = UnsafeCell::new(password);
        let service = UnsafeCell::new("mk");

        // Create the pam conversation structure, holding a callback to our pam conversation function.
        let pam_conv = PamConversation {
            conv: Some(pam_conversation),
            data_ptr: std::ptr::null_mut(),
        };

        // Start the pam conversation.
        let ret = unsafe {
            pam_start(
                service.get() as *mut c_char,
                username.get() as *mut c_char,
                &pam_conv,
                std::ptr::null_mut(),
            )
        };

        if ret != PamReturnCode::SUCCESS as c_int {
            eprintln!("Unable to start PAM {}", ret);
            return Err(());
        }

        println!("Started PAM session!");
        Ok(())
    }
}
