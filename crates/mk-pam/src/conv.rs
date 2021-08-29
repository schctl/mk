//! PAM conversation types.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::sync::Mutex;

use lazy_static::lazy_static;
use libc::{c_char, c_int, c_void};

use crate::ffi;
use crate::prelude::*;

lazy_static! {
    /// Global conversation function pointers.
    ///
    /// A library calling `start` must provide a [`conv::Conversation`].
    /// This needs to be re-exported as an `extern "C" fn`, and needs to be
    /// provided in a [`ffi::pam_conv`].
    ///
    /// The created [`ffi::pam_conv`] will hold a pointer which will be provided to the
    /// exported conversation function. We handle the pointer internally, and use that as
    /// a key to stored global [`conv::Conversation`]s.
    pub(crate) static ref GLOBAL_CONV_PTRS: Mutex<HashMap<c_int, Conversation>> = Mutex::new(HashMap::new());
}

/// Contains the PAM conversation function. This will be called by a loaded PAM module.
pub struct Conversation {
    /// Unlike the regular PAM conversation function, this is called for every message provided.
    pub conv: Box<dyn Fn(&Message) -> Option<Response>>,
}

unsafe impl Send for Conversation {}
unsafe impl Sync for Conversation {}

/// A PAM message.
#[derive(Debug, Clone)]
pub enum Message {
    /// Obtain a string without echoing any text.
    Prompt(String),
    /// Obtain a string while echoing some text.
    PromptEcho(String),
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
            ffi::PAM_PROMPT_ECHO_ON => Ok(Self::PromptEcho(msg)),
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

/// A response to a PAM message.
#[derive(Debug, Clone)]
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

/// Exported PAM conversation function.
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
/// * `response` - Pointer to array of [`PamResponse`].
/// * `appdata_ptr` - Set to the second element of the structure this function was provided in.
///   In our case, the value points to an invalid memory location, but we treat the pointer as
///   a number, and use that to index [`GLOBAL_CONV_PTRS`].
pub(crate) extern "C" fn __raw_pam_conv(
    num_msgs: c_int,
    raw_msgs: *mut *const ffi::pam_message,
    raw_responses: *mut *mut ffi::pam_response,
    appdata_ptr: *mut c_void,
) -> c_int {
    // Lookup if a conversation function is available
    if let Some(f) = GLOBAL_CONV_PTRS
        .lock()
        .unwrap()
        // Interpret pointer's raw value as a number and
        // use that as index.
        .get(&(appdata_ptr as c_int))
    {
        let mut responses = Vec::new();

        for i in 0..num_msgs as isize {
            let raw_msg = unsafe { *raw_msgs.offset(i) };

            // Create message
            let contents = match unsafe { util::cstr_to_string((*raw_msg).msg) } {
                Ok(s) => s,
                // I DON'T KNOW IF THESE RETURN CODES ARE CORRECT
                // (but it should be fine for now)
                Err(_) => return RawError::Buffer.into(),
            };

            let msg = match unsafe { (*raw_msg).msg_style as u32 } {
                ffi::PAM_PROMPT_ECHO_OFF => Message::Prompt(contents),
                ffi::PAM_PROMPT_ECHO_ON => Message::PromptEcho(contents),
                ffi::PAM_TEXT_INFO => Message::ShowText(contents),
                ffi::PAM_ERROR_MSG => Message::ShowError(contents),
                // Error code - SAME HERE
                _ => return RawError::BadItem.into(),
            };

            // Get response and write it
            if let Some(resp) = (f.conv)(&msg) {
                responses.push(match ffi::pam_response::try_from(resp) {
                    Ok(r) => r,
                    // Error code - SAME HERE
                    Err(_) => return RawError::BadItem.into(),
                })
            }
        }

        unsafe { *raw_responses = responses.into_raw_parts().0 };

        return ffi::PAM_SUCCESS as c_int;
    };

    RawError::BadItem.into()
}
