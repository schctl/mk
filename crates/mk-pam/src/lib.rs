//! Raw interface to PAM (Pluggable Authentication Modules).
//!
//! This crate provides **experimental** safe interfaces to PAM as well.

#![feature(vec_into_raw_parts)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::os::raw::{c_int, c_void};
use std::sync::atomic::AtomicI32;
use std::{convert::TryFrom, ffi::CString};

use mk_common::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub mod conv;
pub mod errors;

pub use errors::*;

/// Raw bindings to PAM headers.
pub mod ffi {
    #![allow(non_snake_case)]
    #![allow(non_camel_case_types)]
    #![allow(non_upper_case_globals)]

    include!(concat!(env!("OUT_DIR"), "/ffi.rs"));
}

// See http://uw714doc.sco.com/en/SEC_pam/pam_appl-3.html

/// PAM item types.
#[derive(IntoPrimitive, TryFromPrimitive, Debug, Clone)]
#[repr(i32)]
pub enum ItemType {
    /// The name of the requesting service.
    Service = ffi::PAM_SERVICE as i32,
    /// The name of the user that the application is trying to authenticate.
    User = ffi::PAM_USER as i32,
    /// The string used when prompting for a user's name.
    UserPrompt = ffi::PAM_USER_PROMPT as i32,
    /// The terminal name.
    ///
    /// The name must be prefixed by /dev/ if it is a device file. For graphical, X-based,
    /// applications the value for this item should be the $DISPLAY variable.
    Tty = ffi::PAM_TTY as i32,
    /// The name of the user requesting authentication (the applicant).
    ///
    /// Local name for a locally requesting user or a remote user name for a remote requesting user.
    /// `RequestUser@RequestHost` should always identify the requesting user.
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

/// Information corresponding to an [`ItemType`].
pub enum Item {
    /// See [`ItemType::Service`].
    Service(String),
    /// See [`ItemType::User`].
    User(String),
    /// See [`ItemType::UserPrompt`].
    UserPrompt(String),
    /// See [`ItemType::Tty`].
    Tty(String),
    /// See [`ItemType::RequestUser`].
    RequestUser(String),
    /// See [`ItemType::RequestHost`].
    RequestHost(String),
    /// See [`ItemType::AuthToken`].
    AuthToken(String),
    /// See [`ItemType::OldAuthToken`].
    OldAuthToken(String),
    /// See [`ItemType::Conversation`].
    Conversation(conv::Conversation),
}

#[derive(IntoPrimitive, TryFromPrimitive, Debug, Clone)]
#[repr(i32)]
pub enum Flags {
    /// Do not emit any messages.
    Silent = ffi::PAM_SILENT as i32,
    /// Fail if the user does not have an authentication token.
    DissallowNoAuthToken = ffi::PAM_DISALLOW_NULL_AUTHTOK as i32,
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
    /// *This is a safe interface to [`ffi::pam_start`]. To see more, here are a few links:*
    /// - <http://uw714doc.sco.com/en/SEC_pam/pam_appl-3.html>
    /// - <https://linux.die.net/man/3/pam_start>
    /// - <https://docs.oracle.com/cd/E88353_01/html/E37847/pam-start-3pam.html>
    pub fn start(
        service_name: &str,
        user_name: &str,
        conversation: conv::Conversation,
    ) -> PamResult<Self> {
        let service_name = CString::new(service_name)?;
        let user_name = CString::new(user_name)?;

        let conv = {
            // Insert conversation into the global conversation map
            // with its id as the index in the map.
            let mut global_ptr_lock = conv::GLOBAL_CONV_PTRS.lock().unwrap();
            let mut index = global_ptr_lock.len() as c_int;
            while global_ptr_lock.contains_key(&index) {
                index += 1
            }
            global_ptr_lock.insert(index, conversation);

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

        if pamh.is_null() {
            return Err(FfiError::InvalidPtr.into());
        };

        match RawError::try_from(ret as i32) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(Self {
                interior: pamh,
                last_retcode: AtomicI32::new(ret),
            }),
        }
    }

    /// Terminates the PAM transaction and destroys the corresponding PAM context.
    ///
    /// This is the last function an application should call for this context, and free all
    /// resources allocated to it.
    ///
    /// *This function is a safe interface to [`ffi::pam_end`]. To see more, here are a few links:*
    /// - <https://linux.die.net/man/3/pam_end>
    /// - <https://docs.oracle.com/cd/E88353_01/html/E37847/pam-end-3pam.html>
    pub fn end(self) -> PamResult<()> {
        let ret =
            unsafe { ffi::pam_end(self.interior, self.last_retcode.into_inner() as c_int) } as i32;

        match RawError::try_from(ret) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(()),
        }
    }

    /// Access and update information of a PAM item type.
    ///
    /// *This function is a safe interface to [`ffi::pam_set_item`]. To see more, here are a few links:*
    /// - <https://linux.die.net/man/3/pam_set_item>
    /// - <https://docs.oracle.com/cd/E88353_01/html/E37847/pam-set-item-3pam.html>
    pub fn set_item(&self, item: Item) -> PamResult<()> {
        let (item_ty, item): (c_int, *const c_void) = match item {
            Item::Service(s) => (
                ItemType::Service as c_int,
                CString::new(s)?.as_ptr() as *const c_void,
            ),
            Item::User(s) => (
                ItemType::User as c_int,
                CString::new(s)?.as_ptr() as *const c_void,
            ),
            Item::UserPrompt(s) => (
                ItemType::UserPrompt as c_int,
                CString::new(s)?.as_ptr() as *const c_void,
            ),
            Item::Tty(s) => (
                ItemType::Tty as c_int,
                CString::new(s)?.as_ptr() as *const c_void,
            ),
            Item::RequestUser(s) => (
                ItemType::RequestUser as c_int,
                CString::new(s)?.as_ptr() as *const c_void,
            ),
            Item::RequestHost(s) => (
                ItemType::RequestHost as c_int,
                CString::new(s)?.as_ptr() as *const c_void,
            ),
            Item::AuthToken(s) => (
                ItemType::AuthToken as c_int,
                CString::new(s)?.as_ptr() as *const c_void,
            ),
            Item::OldAuthToken(s) => (
                ItemType::OldAuthToken as c_int,
                CString::new(s)?.as_ptr() as *const c_void,
            ),
            Item::Conversation(s) => {
                // Add conversation to the global conversation map.
                let mut global_ptr_lock = conv::GLOBAL_CONV_PTRS.lock().unwrap();
                let mut index = global_ptr_lock.len() as c_int;
                while global_ptr_lock.contains_key(&index) {
                    index += 1
                }
                global_ptr_lock.insert(index as c_int, s);

                let conv = ffi::pam_conv {
                    conv: Some(conv::__raw_pam_conv),
                    appdata_ptr: { index as *mut c_void },
                };

                (
                    ItemType::Conversation as c_int,
                    &conv as *const ffi::pam_conv as *const c_void,
                )
            }
        };

        let ret = unsafe { ffi::pam_set_item(self.interior, item_ty, item) } as i32;

        self.last_retcode
            .store(ret, std::sync::atomic::Ordering::SeqCst);

        match RawError::try_from(ret) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(()),
        }
    }

    // TODO: `pam_get_item`

    /// Attempts to authenticate the user associated with this handle.
    ///
    /// The user is required to provide an authentication token depending on the authentication
    /// service, usually a password or fingerprint. An [`Err`] is returned if authentication
    /// of the user failed.
    ///
    /// The application is free to call this function as many times as it wishes, but some modules
    /// may maintain an internal retry counter an return [`RawError::MaxTries`] when it reaches a
    /// defined limit.
    ///
    /// The PAM service module may request that the user enter their username via the conversation
    /// mechanism (see [`Handle::start`](crate::Handle::start) and [`conv::Conversation`]).
    /// The name of the authenticated user will be present in the PAM item PAM_USER. This item may
    /// be recovered with a call to [`ffi::pam_get_item`].
    ///
    /// *This function is a safe interface to [`ffi::pam_authenticate`]. To see more, here are a few links:*
    /// - <https://linux.die.net/man/3/pam_authenticate>
    /// - <https://docs.oracle.com/cd/E88353_01/html/E37847/pam-authenticate-3pam.html>
    pub fn authenticate(&self, flags: Option<i32>) -> PamResult<()> {
        let ret = unsafe {
            ffi::pam_authenticate(
                self.interior,
                match flags {
                    Some(f) => f,
                    None => ffi::PAM_SUCCESS as i32,
                } as c_int,
            )
        } as i32;

        self.last_retcode
            .store(ret, std::sync::atomic::Ordering::SeqCst);

        match RawError::try_from(ret) {
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
    /// *This is a safe interface to [`ffi::pam_acct_mgmt`]. To see more, here are a few links:*
    /// - <https://linux.die.net/man/3/pam_acct_mgmt>
    /// - <https://docs.oracle.com/cd/E36784_01/html/E36878/pam-acct-mgmt-3pam.html>
    pub fn validate(&self, flags: Option<i32>) -> PamResult<()> {
        let ret = unsafe {
            ffi::pam_acct_mgmt(
                self.interior,
                match flags {
                    Some(f) => f,
                    None => ffi::PAM_SUCCESS as i32,
                } as c_int,
            )
        } as i32;

        self.last_retcode
            .store(ret, std::sync::atomic::Ordering::SeqCst);

        match RawError::try_from(ret) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(()),
        }
    }

    /// Change the authentication token of the user associated with this handle.
    ///
    /// *This function is a safe interface to [`ffi::pam_chauthtok`]. To see more, here are a few links:*
    /// - <https://linux.die.net/man/3/pam_chauthtok>
    /// - <https://docs.oracle.com/cd/E86824_01/html/E54770/pam-chauthtok-3pam.html>
    pub fn change_auth_token(&self, flags: Option<i32>) -> PamResult<()> {
        let ret = unsafe {
            ffi::pam_chauthtok(
                self.interior,
                match flags {
                    Some(f) => f,
                    None => ffi::PAM_SUCCESS as i32,
                } as c_int,
            )
        } as i32;

        self.last_retcode
            .store(ret, std::sync::atomic::Ordering::SeqCst);

        match RawError::try_from(ret) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(()),
        }
    }
}
