//! PAM conversation types.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::os::raw::{c_int, c_void};
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::*;

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
    pub conv: Box<dyn Fn(&mut [MessageContainer]) -> core::result::Result<(), PamError>>,
}

impl std::fmt::Debug for Conversation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PAM conversation function")
    }
}

unsafe impl Send for Conversation {}
unsafe impl Sync for Conversation {}

/// Structure used in a PAM conversation, containing the message sent from a module,
/// and its corresponding response, if any.
#[readonly::make]
#[derive(Debug)]
pub struct MessageContainer {
    pub msg: Message,
    resp: Option<Response>,
}

impl MessageContainer {
    #[must_use]
    pub fn new(msg: Message) -> Self {
        Self { msg, resp: None }
    }

    /// Get the currently set response to the internal message.
    #[inline]
    pub fn get_response(&self) -> &Option<Response> {
        &self.resp
    }

    /// Set a new response to the internal message.
    #[inline]
    pub fn set_response(&mut self, resp: Option<Response>) {
        self.resp = resp;
    }

    /// Return internal message and response.
    #[inline]
    pub fn into_raw_parts(self) -> (Message, Option<Response>) {
        (self.msg, self.resp)
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
        // Collect messages
        let mut messages = Vec::with_capacity(num_msgs as usize);

        for i in 0..num_msgs as isize {
            messages.push(MessageContainer::new(
                match unsafe { (*raw_msgs).offset(i) }.try_into() {
                    Ok(m) => m,
                    Err(_) => return PamError::Conversation.into(),
                },
            ));
        }

        // Call provided conversation function
        if let Err(e) = (f.conv)(&mut messages[..]) {
            return e.into();
        };

        // Write responses
        let mut responses = Vec::with_capacity(num_msgs as usize);

        for m in messages {
            responses.push(match m.into_raw_parts().1 {
                Some(m) => match ffi::pam_response::try_from(m) {
                    Ok(r) => r,
                    Err(_) => return PamError::Conversation.into(),
                },
                None => unsafe { std::mem::zeroed() },
            })
        }

        unsafe { *raw_responses = responses.into_raw_parts().0 };
    }

    ffi::PAM_SUCCESS as c_int
}
