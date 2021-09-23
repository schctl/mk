use std::ffi::CString;
/// Other PAM types.
use std::io;
use std::os::raw::{c_char, c_int, c_void};

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::errors::*;
use crate::ffi;
use crate::Handle;

/// PAM item types.
#[derive(IntoPrimitive, TryFromPrimitive, Debug, Clone, Copy)]
#[repr(i32)]
#[non_exhaustive]
enum ItemKind {
    /// The name of the requesting service.
    Service = ffi::PAM_SERVICE as i32,
    /// ...
    User = ffi::PAM_USER as i32,
    /// The string used when prompting for a user's name.
    UserPrompt = ffi::PAM_USER_PROMPT as i32,
    /// The terminal name.
    ///
    /// The name must be prefixed by `/dev/` if it is a device file. For graphical, X-based,
    /// applications the value for this item should be the `DISPLAY` environment variable.
    Tty = ffi::PAM_TTY as i32,
    /// ...
    RequestUser = ffi::PAM_RUSER as i32,
    /// The name of the applicant's host machine.
    RequestHost = ffi::PAM_RHOST as i32,
    /// The authentication token.
    AuthToken = ffi::PAM_AUTHTOK as i32,
    /// The old authentication token.
    OldAuthToken = ffi::PAM_OLDAUTHTOK as i32,
    /// The conversation structure.
    Conversation = ffi::PAM_CONV as i32,
    // TODO: Linux-PAM specific items
}

/// Manages PAM items associated with a [`Handle`].
pub struct Items<'a> {
    pub(crate) handle: &'a mut Handle,
}

impl<'a> Items<'a> {
    #[inline]
    fn set_str(&mut self, ty: c_int, data: &str) -> Result<()> {
        self.handle
            .set_item(ty, CString::new(data)?.into_raw() as *const c_void)
    }

    /// The name of the user that will use this service.
    #[inline]
    pub fn set_user(&mut self, user: &str) -> Result<()> {
        self.set_str(ffi::PAM_USER as c_int, user)
    }

    /// The name of the user requesting authentication (the applicant).
    ///
    /// Local name for a locally requesting user or a remote user name for a remote requesting user.
    /// `RequestUser@RequestHost` should always identify the requesting user.
    #[inline]
    pub fn set_request_user(&mut self, user: &str) -> Result<()> {
        self.set_str(ffi::PAM_RUSER as c_int, user)
    }

    /// The name of the applicant's host machine.
    #[inline]
    pub fn set_request_host(&mut self, host: &str) -> Result<()> {
        self.set_str(ffi::PAM_RHOST as c_int, host)
    }
}

bitflags::bitflags! {
    /// General PAM flags.
    pub struct Flags: i32 {
        /// No flags.
        const NONE = 0_i32;
        /// Do not emit any messages.
        const SILENT = ffi::PAM_SILENT as i32;
        /// Fail if the user does not have an authentication token.
        const DISALLOW_NO_AUTH_TOKEN = ffi::PAM_DISALLOW_NULL_AUTHTOK as i32;
        /// This argument indicates to the modules that the user's authentication token (password) should only be changed if it has expired.
        const CHANGE_EXPIRED_AUTH_TOKEN = ffi::PAM_CHANGE_EXPIRED_AUTHTOK as i32;
        /// Initialize the credentials for the user.
        const ESTABLISH_CREDS = ffi::PAM_ESTABLISH_CRED as i32;
        /// Initialize the credentials for the user.
        const DELETE_CREDS = ffi::PAM_DELETE_CRED as i32;
        /// Initialize the credentials for the user.
        const REINITIALIZE_CREDS = ffi::PAM_REINITIALIZE_CRED as i32;
        /// Initialize the credentials for the user.
        const REFRESH_CREDS = ffi::PAM_REFRESH_CRED as i32;
    }
}

/// A PAM message.
#[derive(IntoPrimitive, TryFromPrimitive, Debug, Clone, Copy)]
#[repr(i32)]
#[non_exhaustive]
pub enum MessageType {
    /// Obtain a string while echoing some text.
    Prompt = ffi::PAM_PROMPT_ECHO_ON as i32,
    /// Obtain a string without echoing any text.
    PromptNoEcho = ffi::PAM_PROMPT_ECHO_OFF as i32,
    /// Display an error message.
    ShowError = ffi::PAM_ERROR_MSG as i32,
    /// Display some text.
    ShowText = ffi::PAM_TEXT_INFO as i32,
}

/// A PAM message.
#[derive(Debug)]
pub struct Message {
    /// The actual message.
    contents: String,
    /// The type of message.
    kind: MessageType,
}

impl Message {
    /// Create a new PAM message.
    #[must_use]
    pub fn new(contents: String, kind: MessageType) -> Self {
        Self { contents, kind }
    }

    #[must_use]
    #[inline]
    pub fn contents(&self) -> &String {
        &self.contents
    }

    #[must_use]
    #[inline]
    pub fn kind(&self) -> MessageType {
        self.kind
    }
}

impl TryFrom<*const ffi::pam_message> for Message {
    type Error = Error;

    /// Convert a raw *[`ffi::pam_message`] to a [`Message`]. Returns the
    /// message contents as a [`String`] if it is of an unknown type.
    fn try_from(value: *const ffi::pam_message) -> Result<Self> {
        if value.is_null() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "null pointer").into());
        }

        let value = unsafe { *value };
        let contents = unsafe { mk_common::chars_to_string(value.msg as *mut c_char)? };

        Ok(Self {
            contents,
            kind: match value.msg_style.try_into() {
                Ok(k) => k,
                Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e).into()),
            },
        })
    }
}

/// A response to a PAM message.
#[derive(Debug, Clone)]
pub struct Response {
    /// The actual response.
    pub resp: String,
}

impl TryFrom<Response> for ffi::pam_response {
    type Error = Error;

    fn try_from(value: Response) -> Result<Self> {
        Ok(ffi::pam_response {
            resp: CString::new(value.resp)?.into_raw(),
            resp_retcode: 0,
        })
    }
}

/// Structure used in a PAM conversation, containing the message sent from a module,
/// and its corresponding response, if any.
#[derive(Debug)]
pub struct MessageContainer {
    pub msg: Message,
    pub resp: Option<Response>,
}

impl MessageContainer {
    #[must_use]
    pub fn new(msg: Message) -> Self {
        Self { msg, resp: None }
    }
}
