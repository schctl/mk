//! Rust interface to `shadow.h` provided by [`shadow-utils`].
//!
//! See also [`shadow(3)`](https://www.man7.org/linux/man-pages/man3/shadow.3.html).
//!
//! [`shadow-utils`]: <https://github.com/shadow-maint/shadow>

use std::ffi::CString;
use std::io;
use std::time;

use mk_common::*;

pub type Uid = libc::uid_t;
pub type Gid = libc::gid_t;

/// Raw bindings to shadow headers.
mod ffi {
    #![allow(unused)]
    #![allow(non_snake_case)]
    #![allow(non_camel_case_types)]
    #![allow(non_upper_case_globals)]
    #![allow(clippy::upper_case_acronyms)]
    #![allow(clippy::redundant_static_lifetimes)]

    include!(concat!(env!("OUT_DIR"), "/ffi.rs"));
}

/// A single entry in the shadow file.
///
/// See <https://linux.die.net/man/3/shadow> for more.
#[derive(Debug, Clone)]
pub struct Spwd {
    /// User's login name.
    pub name: String,
    /// Encrypted password.
    pub password: String,
    /// Date of last password change.
    pub date_change: std::time::SystemTime,
    /// Minimum password age.
    ///
    /// The duration that must pass before the user's password can be changed.
    pub password_age_min: time::Duration,
    /// Maximum password age.
    ///
    /// The duration after which the user's password must be changed.
    pub password_age_max: time::Duration,
    /// Warning period.
    ///
    /// The duration before password expiry, in which the user will be warned to change their password.
    pub password_warn_duration: time::Duration,
    /// Inactive period.
    ///
    /// The duration after password expiry, after which the user's account will be disabled.
    pub password_inactive_duration: time::Duration,
    /// Date when the user's account expired.
    pub expiry: time::SystemTime,
    /// Reserved field.
    pub flag: u64,
}

impl Spwd {
    /// Get a [`Spwd`] struct from a raw [`libc::spwd`] pointer.
    ///
    /// # Errors
    ///
    /// - [`io::Error`] of kind [`io::ErrorKind::InvalidData`] - when the pointer is null.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid pointer to a [`libc::spwd`].
    pub unsafe fn from_raw(ptr: *mut libc::spwd) -> io::Result<Self> {
        if ptr.is_null() {
            io_bail!(InvalidData, "null pointer");
        }

        let raw = *ptr;

        Ok(Self {
            name: cstr_to_string(raw.sp_namp)?,
            password: cstr_to_string(raw.sp_pwdp)?,
            date_change: time::UNIX_EPOCH
                + time::Duration::from_secs((raw.sp_lstchg as u64) * 86_400),
            password_age_min: time::Duration::from_secs((raw.sp_min as u64) * 86_400),
            password_age_max: time::Duration::from_secs((raw.sp_max as u64) * 86_400),
            password_warn_duration: time::Duration::from_secs((raw.sp_warn as u64) * 86_400),
            password_inactive_duration: time::Duration::from_secs((raw.sp_inact as u64) * 86_400),
            expiry: time::UNIX_EPOCH + time::Duration::from_secs((raw.sp_expire as u64) * 86_400),
            flag: raw.sp_flag as u64,
        })
    }

    /// Get a [`Spwd`] entry from a user name.
    ///
    /// # Errors
    ///
    /// - [`io::Error`] if a user was not found or if an error occurred while processing.
    pub fn from_name(name: &str) -> io::Result<Self> {
        unsafe { Self::from_raw(libc::getspnam(CString::new(name)?.as_ptr())) }
    }

    /// Get a [`Spwd`] entry from a [`mk_pwd::Passwd`] entry.
    ///
    /// # Errors
    ///
    /// - [`io::Error`] if a user was not found or if an error occurred while processing.
    pub fn from_passwd(pwd: &mk_pwd::Passwd) -> io::Result<Self> {
        unsafe { Self::from_raw(libc::getspnam(CString::new(&pwd.name[..])?.as_ptr())) }
    }
}
