//! PAM conversation handling.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::os::raw::{c_int, c_void};
use std::sync::Mutex;

use crate::*;

lazy_static::lazy_static! {
    /// Global conversation function pointers.
    ///
    /// A library calling `start` must provide a [`conv::Conversation`].
    /// This needs to be re-exported as an `extern "C" fn`, and needs to be
    /// provided in a [`ffi::pam_conv`].
    ///
    /// The created [`ffi::pam_conv`] will hold a pointer which will be provided to the
    /// exported conversation function. We handle the pointer internally, and use that as
    /// a key to stored global [`conv::Conversation`]s.
    static ref GLOBAL_CONV_PTRS: Mutex<HashMap<usize, Conversation>> = Mutex::new(HashMap::new());
}

/// PAM conversation function. This will be called by a loaded PAM module.
pub type ConversationCallback =
    Box<dyn Fn(&mut [MessageContainer]) -> core::result::Result<(), PamError>>;

/// Container for a PAM conversation.
pub(crate) struct Conversation {
    conv: ConversationCallback,
}

impl Conversation {
    /// Store a new conversation in the global conversation map.
    pub fn add(conv: ConversationCallback) -> usize {
        let mut global_ptr_lock = conv::GLOBAL_CONV_PTRS.lock().unwrap();
        let mut index = 0;
        while global_ptr_lock.contains_key(&index) {
            index += 1
        }
        let conv = Self { conv };
        global_ptr_lock.insert(index, conv);
        index
    }

    pub fn remove(index: usize) -> Option<ConversationCallback> {
        conv::GLOBAL_CONV_PTRS
            .lock()
            .unwrap()
            .remove(&index)
            .map(|c| c.conv)
    }
}

unsafe impl Send for Conversation {}
unsafe impl Sync for Conversation {}

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
        .get(&(appdata_ptr as c_int as usize))
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
            responses.push(match m.resp {
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
