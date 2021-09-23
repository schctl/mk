//! Rust interface to PAM (Pluggable Authentication Modules).
//!
//! ## Feature flags
//!
//! - `linux-pam`: Support for [`Linux-PAM`] extensions.
//! - `open-pam`: Support for [`OpenPAM`] extensions.
//!
//! ## Read more:
//!
//! - [RHEL docs](https://web.mit.edu/rhel-doc/5/RHEL-5-manual/Deployment_Guide-en-US/ch-pam.html)
//! - [Oracle docs](https://docs.oracle.com/cd/E23824_01/html/821-1456/pam-2.html)
//! - [IBM docs](https://www.ibm.com/docs/en/aix/7.2?topic=system-pluggable-authentication-modules)
//! - [The Linux-PAM Application Developers' Guide](http://uw714doc.sco.com/en/SEC_pam/pam_appl.html)
//! - [Wikipedia](https://en.wikipedia.org/wiki/Pluggable_authentication_module)
//!
//! [`OpenPAM`]: https://www.openpam.org/wiki
//! [`Linux-PAM`]: http://www.linux-pam.org/

#![feature(vec_into_raw_parts)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::io;
use std::os::raw::{c_int, c_void};
use std::{convert::TryFrom, ffi::CString};

mod conv;
mod errors;
mod types;

pub use conv::*;
pub use errors::*;
pub use types::*;

/// Raw bindings to PAM headers.
pub mod ffi {
    #![allow(clippy::all, clippy::pedantic)]
    #![allow(unused, non_snake_case, non_camel_case_types, non_upper_case_globals)]
    include!(concat!(env!("OUT_DIR"), "/ffi.rs"));
}

// See http://uw714doc.sco.com/en/SEC_pam/pam_appl-3.html

/// Represents a PAM handle.
///
/// This provides safe interfaces to standard PAM functions as well. Documentation and links
/// for each function is provided. You can also find documentation for each function from the `man`
/// page of the corresponding [`ffi`] function.
pub struct Handle {
    interior: *mut ffi::pam_handle,
    last_retcode: c_int,
    index: usize,
}

unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}

impl Handle {
    /// Creates the *PAM context* and initializes the PAM transaction.
    ///
    /// This is the first of the PAM functions that must be called by an application.
    /// It initializes the interface and reads the system configuration file, `/etc/pam.conf`.
    ///
    /// # Errors
    ///
    /// - Error of type [`Error::Io`] if the provided user name or service name contained an interior
    /// nul-byte.
    /// - Error of type [`Error::Raw`] if the underlying PAM call failed, or if the returned pointer
    /// was invalid.
    ///
    /// # Read more
    ///
    /// *This is a safe interface to [`ffi::pam_start`]*.
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

        let index = conv::Conversation::add(conversation);

        let conv = ffi::pam_conv {
            conv: Some(conv::__raw_pam_conv),
            // This is a very hack-y approach.
            // This is actually an invalid pointer, but in `__raw_pam_conv`
            // we treat the pointer's address as a number, and use that as an index.
            appdata_ptr: { index as *mut c_void },
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
            last_retcode: ret,
            index,
        })
    }

    /// Terminates the PAM transaction and destroys the corresponding PAM context.
    ///
    /// This is the last function an application should call for this context. Frees all
    /// resources allocated to this handle.
    ///
    /// # Read more
    ///
    /// *This function is a safe interface to [`ffi::pam_end`]*.
    /// - <https://linux.die.net/man/3/pam_end>
    /// - <https://docs.oracle.com/cd/E88353_01/html/E37847/pam-end-3pam.html>
    #[inline]
    pub fn end(self) {
        // drop
    }

    /// Access and update information of a PAM item type.
    ///
    /// # Errors
    ///
    /// All errors returned by this call are [`Error::Raw`].
    ///
    /// # Read more
    ///
    /// *This function is a safe interface to [`ffi::pam_set_item`]*.
    /// - <https://linux.die.net/man/3/pam_set_item>
    /// - <https://docs.oracle.com/cd/E88353_01/html/E37847/pam-set-item-3pam.html>
    pub(crate) fn set_item(&mut self, kind: c_int, item: *const c_void) -> Result<()> {
        let ret = unsafe { ffi::pam_set_item(self.interior, kind, item) } as i32;

        self.last_retcode = ret;

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
    /// let mut handle = Handle::start("foo", "bar", Box::new(|_| Ok(()))).unwrap();
    /// handle.items().set_user("more_foo").unwrap();
    /// ```
    #[must_use]
    #[inline]
    pub fn items(&mut self) -> Items<'_> {
        Items { handle: self }
    }

    // TODO: `pam_get_item`

    /// Attempts to authenticate the user associated with this handle.
    ///
    /// The user is required to provide an authentication token depending on the authentication
    /// service, usually a password or fingerprint. An [`Err`] is returned if authentication
    /// of the user failed.
    ///
    /// The PAM service module may request that the user enter their username via the conversation
    /// mechanism (see [`start`] and [`ConversationCallback`]).
    /// The name of the authenticated user will be present in the PAM user item (see [`set_user`]).
    ///
    /// # Errors
    ///
    /// All errors returned by this call are [`Error::Raw`]. Additionally, the application is free to
    /// call this function as many times as it wishes, but some modules may maintain an internal retry
    /// counter, and return [`PamError::MaxTries`] when it reaches a pre-defined limit.
    ///
    /// # Read more
    ///
    /// *This function is a safe interface to [`ffi::pam_authenticate`]*.
    /// - <https://linux.die.net/man/3/pam_authenticate>
    /// - <https://docs.oracle.com/cd/E88353_01/html/E37847/pam-authenticate-3pam.html>
    ///
    /// [`start`]: Self::start
    /// [`set_user`]: Items::set_user
    pub fn authenticate(&mut self, flags: Flags) -> Result<()> {
        let ret = unsafe { ffi::pam_authenticate(self.interior, flags.bits() as c_int) } as i32;

        self.last_retcode = ret;

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
    /// # Errors
    ///
    /// All errors returned by this call are [`Error::Raw`].
    ///
    /// # Read more
    ///
    /// *This is a safe interface to [`ffi::pam_acct_mgmt`]*.
    /// - <https://linux.die.net/man/3/pam_acct_mgmt>
    /// - <https://docs.oracle.com/cd/E36784_01/html/E36878/pam-acct-mgmt-3pam.html>
    pub fn validate(&mut self, flags: Flags) -> Result<()> {
        let ret = unsafe { ffi::pam_acct_mgmt(self.interior, flags.bits() as c_int) } as i32;

        self.last_retcode = ret;

        match PamError::try_from(ret) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(()),
        }
    }

    /// Change the authentication token of the user associated with this handle.
    ///
    /// # Errors
    ///
    /// All errors returned by this call are [`Error::Raw`].
    ///
    /// # Read more
    ///
    /// *This function is a safe interface to [`ffi::pam_chauthtok`]*.
    /// - <https://linux.die.net/man/3/pam_chauthtok>
    /// - <https://docs.oracle.com/cd/E86824_01/html/E54770/pam-chauthtok-3pam.html>
    pub fn change_auth_token(&mut self, flags: Flags) -> Result<()> {
        let ret = unsafe { ffi::pam_chauthtok(self.interior, flags.bits() as c_int) } as i32;

        self.last_retcode = ret;

        match PamError::try_from(ret) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(()),
        }
    }

    /// Start a user session.
    ///
    /// This function is used to set up a user session for a previously authenticated user, and informs
    /// modules that a session has begun. The session should be terminated with a call to
    /// [`close_session`]. An [`Err`] is returned if a session could not be opened.
    ///
    /// # Errors
    ///
    /// All errors returned by this call are [`Error::Raw`].
    ///
    /// # Read more
    ///
    /// *This function is a safe interface to [`ffi::pam_open_session`]*.
    /// - <https://linux.die.net/man/3/pam_open_session>
    /// - <https://docs.oracle.com/cd/E36784_01/html/E36878/pam-open-session-3pam.html>
    ///
    /// [`close_session`]: Self::close_session
    pub fn open_session(&mut self, flags: Flags) -> Result<()> {
        let ret = unsafe { ffi::pam_open_session(self.interior, flags.bits() as c_int) } as i32;

        self.last_retcode = ret;

        match PamError::try_from(ret) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(()),
        }
    }

    /// Terminate a user session.
    ///
    /// This function is used to indicate that an authenticated session has ended. Read also
    /// [`open_session`].
    ///
    /// # Errors
    ///
    /// All errors returned by this call are [`Error::Raw`].
    ///
    /// # Read more
    ///
    /// *This function is a safe interface to [`ffi::pam_close_session`]*.
    /// - <https://linux.die.net/man/3/pam_close_session>
    /// - <https://docs.oracle.com/cd/E36784_01/html/E36878/pam-close-session-3pam.html>
    ///
    /// [`open_session`]: Self::open_session
    pub fn close_session(&mut self, flags: Flags) -> Result<()> {
        let ret = unsafe { ffi::pam_close_session(self.interior, flags.bits() as c_int) } as i32;

        self.last_retcode = ret;

        match PamError::try_from(ret) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(()),
        }
    }

    /// Establish, maintain, and delete the credentials of a user.
    ///
    /// It should be called to set the credentials after a user has been authenticated and before a
    /// session is opened for the user with [`open_session`]. The credentials should be deleted after
    /// the session has been closed with [`close_session`].
    ///
    /// # Errors
    ///
    /// All errors returned by this call are [`Error::Raw`].
    ///
    /// # Read more
    ///
    /// *This function is a safe interface to [`ffi::pam_close_session`]*.
    /// - <https://linux.die.net/man/3/pam_setcred>
    /// - <https://docs.oracle.com/cd/E36784_01/html/E36878/pam-setcred-3pam.html>
    ///
    /// [`open_session`]: Self::open_session
    /// [`close_session`]: Self::close_session
    pub fn set_creds(&mut self, flags: Flags) -> Result<()> {
        let ret = unsafe { ffi::pam_setcred(self.interior, flags.bits() as c_int) } as i32;

        self.last_retcode = ret;

        match PamError::try_from(ret) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(()),
        }
    }
}

impl Drop for Handle {
    /// See documentation on [`end`].
    ///
    /// [`end`]: Self::end
    fn drop(&mut self) {
        // Usually the only errors that can happen are if the submitted handle is invalid, but we don't
        // allow construction if PAM gives us an invalid handle.
        let _ = unsafe { ffi::pam_end(self.interior, self.last_retcode) };
        conv::Conversation::remove(self.index);
    }
}
