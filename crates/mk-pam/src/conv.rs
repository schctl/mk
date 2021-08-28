//! Pam conversation types.

use std::convert::TryFrom;
use std::ffi::{CStr, CString};

use libc::{c_char, c_int, c_void};

use crate::ffi;

pub type Conversation = extern "C" fn(
    num_msgs: c_int,
    msgs: *mut *const ffi::pam_message,
    response: *mut *mut ffi::pam_response,
    _appdata_ptr: *mut c_void,
) -> c_int;

/// A PAM message.
#[derive(Debug, Clone)]
pub enum Message {
    /// Obtain a string without echoing any text.
    Prompt(String),
    /// Obtain a string while echoing some text.
    PrompEcho(String),
    /// Display an error message.
    ShowError(String),
    /// Display some text.
    ShowText(String),
}

impl TryFrom<ffi::pam_message> for Message {
    type Error = *const c_char;

    /// Convert a raw [`ffi::pam_message`] to a [`Message`]. Returns the
    /// message contents as a [`String`] if it is of an unknown type.
    fn try_from(value: ffi::pam_message) -> Result<Self, Self::Error> {
        let msg = match unsafe { CStr::from_ptr(value.msg) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return Err(value.msg),
        };

        match value.msg_style as u32 {
            ffi::PAM_PROMPT_ECHO_OFF => Ok(Self::Prompt(msg)),
            ffi::PAM_PROMPT_ECHO_ON => Ok(Self::PrompEcho(msg)),
            ffi::PAM_ERROR_MSG => Ok(Self::ShowError(msg)),
            ffi::PAM_TEXT_INFO => Ok(Self::ShowText(msg)),
            _ => Err(value.msg),
        }
    }
}

impl TryFrom<*const ffi::pam_message> for Message {
    type Error = *const c_char;

    /// Convert a raw *[`ffi::pam_message`] to a [`Message`]. Returns the
    /// message contents as a [`String`] if it is of an unknown type.
    fn try_from(value: *const ffi::pam_message) -> Result<Self, Self::Error> {
        Self::try_from(unsafe { *value })
    }
}

/// A response to a Pam message.
pub struct Response {
    /// The actual response.
    pub resp: String,
    /// Unused - 0 is expected.
    pub retcode: i32,
}

impl TryFrom<Response> for ffi::pam_response {
    type Error = std::ffi::NulError;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        Ok(ffi::pam_response {
            resp: CString::new(value.resp)?.into_raw(),
            resp_retcode: value.retcode,
        })
    }
}
