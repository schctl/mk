//! Rust interface to PAM (Pluggable Authentication Modules).
//!
//! ## Feature flags
//!
//! - `linux-pam`: Supported for Linux-PAM extensions.
//! - `open-pam`: Supported for OpenPAM extensions.
//!
//! ## Potentially useful links
//!
//! - [OpenPAM](https://www.openpam.org/wiki>)
//! - [Linux-PAM](http://www.linux-pam.org/)
//! - [RHEL docs](https://web.mit.edu/rhel-doc/5/RHEL-5-manual/Deployment_Guide-en-US/ch-pam.html)
//! - [Oracle docs](https://docs.oracle.com/cd/E23824_01/html/821-1456/pam-2.html)
//! - [IBM docs](https://www.ibm.com/docs/en/aix/7.2?topic=system-pluggable-authentication-modules)
//! - [The Linux-PAM Application Developers' Guide](http://uw714doc.sco.com/en/SEC_pam/pam_appl.html)
//! - [Wikipedia](https://en.wikipedia.org/wiki/Pluggable_authentication_module)

#![feature(vec_into_raw_parts)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::io;
use std::os::raw::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicI32, Ordering};
use std::{convert::TryFrom, ffi::CString};

use num_enum::{IntoPrimitive, TryFromPrimitive};

pub mod conv;
pub mod errors;

pub use errors::*;

/// Raw bindings to PAM headers.
pub mod ffi {
    // ow.
    #![allow(unused, non_snake_case, non_camel_case_types, non_upper_case_globals)]
    #![allow(clippy::upper_case_acronyms, clippy::redundant_static_lifetimes)]

    include!(concat!(env!("OUT_DIR"), "/ffi.rs"));
}

// See http://uw714doc.sco.com/en/SEC_pam/pam_appl-3.html

/// PAM item types.
#[derive(IntoPrimitive, TryFromPrimitive, Debug, Clone, Copy)]
#[repr(i32)]
#[non_exhaustive]
pub(crate) enum ItemKind {
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
    handle: &'a Handle,
}

impl<'a> Items<'a> {
    #[inline(always)]
    fn set_str(&self, data: &str) -> Result<()> {
        self.handle.set_item(
            ItemKind::RequestUser as c_int,
            CString::new(data)?.into_raw() as *const c_void,
        )
    }

    /// The name of the user that the application is trying to authenticate.
    #[inline]
    pub fn set_user(&self, user: &str) -> Result<()> {
        self.set_str(user)
    }

    /// The name of the user requesting authentication (the applicant).
    ///
    /// Local name for a locally requesting user or a remote user name for a remote requesting user.
    /// `RequestUser@RequestHost` should always identify the requesting user.
    #[inline]
    pub fn set_request_user(&self, user: &str) -> Result<()> {
        self.set_str(user)
    }
}

bitflags::bitflags! {
    /// General PAM flags.
    pub struct Flag: i32 {
        /// Do not emit any messages.
        const SILENT = ffi::PAM_SILENT as i32;
        /// Fail if the user does not have an authentication token.
        const DISALLOW_NO_AUTH_TOKEN = ffi::PAM_DISALLOW_NULL_AUTHTOK as i32;
    }
}

/// A PAM message.
#[derive(IntoPrimitive, TryFromPrimitive, Debug, Clone, Copy)]
#[repr(i32)]
#[non_exhaustive]
pub enum MessageType {
    /// Obtain a string without echoing any text.
    Prompt = ffi::PAM_PROMPT_ECHO_OFF as i32,
    /// Obtain a string while echoing some text.
    PromptEcho = ffi::PAM_PROMPT_ECHO_ON as i32,
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

    #[inline]
    pub fn contents(&self) -> &String {
        &self.contents
    }

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
    /// Unused - 0 is expected.
    pub retcode: i64,
}

impl Response {
    /// Create a new PAM message response.
    #[must_use]
    pub fn new(resp: String, retcode: i64) -> Self {
        Self { resp, retcode }
    }
}

impl TryFrom<Response> for ffi::pam_response {
    type Error = Error;

    fn try_from(value: Response) -> Result<Self> {
        Ok(ffi::pam_response {
            resp: CString::new(value.resp)?.into_raw(),
            resp_retcode: value.retcode as c_int,
        })
    }
}

/// Represents a PAM handle.
///
/// This provides safe interfaces to standard PAM functions as well. Documentation and links
/// for each function is provided. You can also find documentation for each function from the `man`
/// page of the corresponding `ffi` function.
pub struct Handle {
    interior: *mut ffi::pam_handle,
    last_retcode: AtomicI32,
}

impl Handle {
    /// Creates the *PAM context* and initializes the PAM transaction.
    ///
    /// This is the first of the PAM functions that must be called by an application.
    /// It initializes the interface and reads the system configuration file, `/etc/pam.conf`.
    /// Following a successful return, the contents of `*pamh` is a handle that provides continuity for
    /// successive calls to the PAM library.
    ///
    /// *This is a safe interface to [`ffi::pam_start`]. To read more, here are a few links:*
    /// - <http://uw714doc.sco.com/en/SEC_pam/pam_appl-3.html>
    /// - <https://linux.die.net/man/3/pam_start>
    /// - <https://docs.oracle.com/cd/E88353_01/html/E37847/pam-start-3pam.html>
    pub fn start(
        service_name: &str,
        user_name: &str,
        conversation: conv::ConversationCallback,
    ) -> Result<Self> {
        let service_name = CString::new(service_name)?;
        let user_name = CString::new(user_name)?;

        let conv = {
            let index = conv::Conversation::add(conversation);

            ffi::pam_conv {
                conv: Some(conv::__raw_pam_conv),
                // This is a very hack-y approach.
                // This is actually an invalid pointer, but in `__raw_pam_conv`
                // we treat the pointer's address as a number, and use that as an index.
                appdata_ptr: { index as *mut c_void },
            }
        };

        let mut pamh: *mut ffi::pam_handle = std::ptr::null_mut();

        let ret =
            unsafe { ffi::pam_start(service_name.as_ptr(), user_name.as_ptr(), &conv, &mut pamh) }
                as i32;

        if let Ok(e) = PamError::try_from(ret as i32) {
            return Err(e.into());
        }

        if pamh.is_null() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "null pointer").into());
        };

        Ok(Self {
            interior: pamh,
            last_retcode: AtomicI32::new(ret),
        })
    }

    /// Terminates the PAM transaction and destroys the corresponding PAM context.
    ///
    /// This is the last function an application should call for this context, and free all
    /// resources allocated to it.
    ///
    /// *This function is a safe interface to [`ffi::pam_end`]. To read more, here are a few links:*
    /// - <https://linux.die.net/man/3/pam_end>
    /// - <https://docs.oracle.com/cd/E88353_01/html/E37847/pam-end-3pam.html>
    pub fn end(self) {
        // drop
    }

    /// Access and update information of a PAM item type.
    ///
    /// *This function is a safe interface to [`ffi::pam_set_item`]. To read more, here are a few links:*
    /// - <https://linux.die.net/man/3/pam_set_item>
    /// - <https://docs.oracle.com/cd/E88353_01/html/E37847/pam-set-item-3pam.html>
    pub(crate) fn set_item(&self, kind: c_int, item: *const c_void) -> Result<()> {
        let ret = unsafe { ffi::pam_set_item(self.interior, kind, item) } as i32;

        self.last_retcode.store(ret, Ordering::SeqCst);

        match PamError::try_from(ret) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(()),
        }
    }

    /// Access and update information of a PAM item type.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use mk_pam::Handle;
    ///
    /// let handle = Handle::start("foo", "more_foo", Box::new(|_| Ok(()))).unwrap();
    ///
    /// handle.items().set_user("more_foo").unwrap();
    /// handle.authenticate(None).unwrap();
    /// ```
    pub fn items(&self) -> Items<'_> {
        Items { handle: self }
    }

    // TODO: `pam_get_item`

    /// Attempts to authenticate the user associated with this handle.
    ///
    /// The user is required to provide an authentication token depending on the authentication
    /// service, usually a password or fingerprint. An [`Err`] is returned if authentication
    /// of the user failed.
    ///
    /// The application is free to call this function as many times as it wishes, but some modules
    /// may maintain an internal retry counter an return [`PamError::MaxTries`] when it reaches a
    /// defined limit.
    ///
    /// The PAM service module may request that the user enter their username via the conversation
    /// mechanism (see [`Handle::start`](crate::Handle::start) and [`conv::Conversation`]).
    /// The name of the authenticated user will be present in the PAM item PAM_USER. This item may
    /// be recovered with a call to [`ffi::pam_get_item`].
    ///
    /// *This function is a safe interface to [`ffi::pam_authenticate`]. To read more, here are a few links:*
    /// - <https://linux.die.net/man/3/pam_authenticate>
    /// - <https://docs.oracle.com/cd/E88353_01/html/E37847/pam-authenticate-3pam.html>
    pub fn authenticate(&self, flags: Option<Flag>) -> Result<()> {
        let ret = unsafe {
            ffi::pam_authenticate(
                self.interior,
                match flags {
                    Some(f) => f,
                    None => Flag::empty(),
                }
                .bits() as c_int,
            )
        } as i32;

        self.last_retcode.store(ret, Ordering::SeqCst);

        match PamError::try_from(ret) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(()),
        }
    }

    /// Validates the account of the user associated with this handle.
    ///
    /// This function checks for the user's authentication token and account expiration, and access
    /// restrictions, and verifies that the user is permitted to gain access to the system at this time.
    /// An [`Err`] is returned if validation of the user's account failed.
    ///
    /// *This is a safe interface to [`ffi::pam_acct_mgmt`]. To read more, here are a few links:*
    /// - <https://linux.die.net/man/3/pam_acct_mgmt>
    /// - <https://docs.oracle.com/cd/E36784_01/html/E36878/pam-acct-mgmt-3pam.html>
    pub fn validate(&self, flags: Option<Flag>) -> Result<()> {
        let ret = unsafe {
            ffi::pam_acct_mgmt(
                self.interior,
                match flags {
                    Some(f) => f,
                    None => Flag::empty(),
                }
                .bits() as c_int,
            )
        } as i32;

        self.last_retcode.store(ret, Ordering::SeqCst);

        match PamError::try_from(ret) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(()),
        }
    }

    /// Change the authentication token of the user associated with this handle.
    ///
    /// *This function is a safe interface to [`ffi::pam_chauthtok`]. To read more, here are a few links:*
    /// - <https://linux.die.net/man/3/pam_chauthtok>
    /// - <https://docs.oracle.com/cd/E86824_01/html/E54770/pam-chauthtok-3pam.html>
    pub fn change_auth_token(&self, flags: Option<Flag>) -> Result<()> {
        let ret = unsafe {
            ffi::pam_chauthtok(
                self.interior,
                match flags {
                    Some(f) => f,
                    None => Flag::empty(),
                }
                .bits() as c_int,
            )
        } as i32;

        self.last_retcode.store(ret, Ordering::SeqCst);

        match PamError::try_from(ret) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(()),
        }
    }

    /// Start a user session.
    ///
    /// This function is used to set up a user session for a previously authenticated user, and informs
    /// modules that a session has begun. The session should be terminated with a call to
    /// [`Handle::close_session`](crate::Handle::close_session). An [`Err`] is returned if a session
    /// could not be opened.
    ///
    /// *This function is a safe interface to [`ffi::pam_open_session`]. To read more, here are a few links:*
    /// - <https://linux.die.net/man/3/pam_open_session>
    /// - <https://docs.oracle.com/cd/E36784_01/html/E36878/pam-open-session-3pam.html>
    pub fn open_session(&self, flags: Option<Flag>) -> Result<()> {
        let ret = unsafe {
            ffi::pam_open_session(
                self.interior,
                match flags {
                    Some(f) => f,
                    None => Flag::empty(),
                }
                .bits() as c_int,
            )
        } as i32;

        self.last_retcode.store(ret, Ordering::SeqCst);

        match PamError::try_from(ret) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(()),
        }
    }

    /// Terminate a user session.
    ///
    /// This function is used to indicate that an authenticated session has ended. See
    /// [`Handle::open_session`](crate::Handle::open_session) for more.
    ///
    /// *This function is a safe interface to [`ffi::pam_close_session`]. To read more, here are a few links:*
    /// - <https://linux.die.net/man/3/pam_close_session>
    /// - <https://docs.oracle.com/cd/E36784_01/html/E36878/pam-close-session-3pam.html>
    pub fn close_session(&self, flags: Option<i32>) -> Result<()> {
        let ret = unsafe {
            ffi::pam_close_session(
                self.interior,
                match flags {
                    Some(f) => f,
                    None => ffi::PAM_SUCCESS as i32,
                } as c_int,
            )
        } as i32;

        self.last_retcode.store(ret, Ordering::SeqCst);

        match PamError::try_from(ret) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(()),
        }
    }
}

impl Drop for Handle {
    /// See documentation on [`Self::end`](crate::Handle::end).
    fn drop(&mut self) {
        // Usually the only errors that can happen are if the submitted handle is invalid, but we don't
        // allow construction if PAM gives us an invalid handle.
        let _ = unsafe {
            ffi::pam_end(
                self.interior,
                self.last_retcode.load(Ordering::SeqCst) as c_int,
            )
        };
    }
}
