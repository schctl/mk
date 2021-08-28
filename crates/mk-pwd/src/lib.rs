//! Interface to Unix's `pwd.h`.

use std::ffi::{CStr, CString, NulError};
use std::str::Utf8Error;

use libc::c_char;
use thiserror::Error;

pub type Uid = libc::uid_t;
pub type Gid = libc::gid_t;

/// All error types that we handle.
#[derive(Error, Debug)]
pub enum PwdError {
    /// An interior nul byte was found.
    #[error("An interior nul byte was found")]
    NulError(#[from] NulError),
    /// Error interpreting byte sequence as utf-8.
    #[error("Error interpreting byte sequence as utf-8")]
    Utf8Error(#[from] Utf8Error),
    /// Null pointer error.
    #[error("Null pointer error")]
    NullPtr,
}

pub type PwdResult<T> = Result<T, PwdError>;

/// Utility to convert from a C *[`c_char`] to a Rust [`String`] safely.
fn cstr_to_string(ptr: *mut c_char) -> PwdResult<String> {
    if ptr.is_null() {
        return Err(PwdError::NullPtr);
    }
    Ok(unsafe { CStr::from_ptr(ptr) }.to_str()?.to_string())
}

/// The `/etc/passwd` file is a text file that describes user login accounts for the system.
/// Each line of the file describes a single user, this struct is a representation of each entry.
///
/// See <https://www.man7.org/linux/man-pages/man5/passwd.5.html> for more.
#[derive(Debug, Clone)]
pub struct Passwd {
    /// User's login name.
    pub name: String,
    /// This is either the encrypted user password, an asterisk (*), or the letter 'x'.
    ///
    /// See <https://www.man7.org/linux/man-pages/man5/group.5.html> for more.
    pub password: Option<String>,
    /// User's unique ID.
    pub uid: Uid,
    /// User's numeric primary group ID.
    pub gid: Gid,
    /// Used for informational purposes, sometimes called the comment field.
    pub gecos: Option<String>,
    /// User's home directory.
    pub directory: String,
    /// Path to user's shell - which is run at login.
    pub shell: String,
}

impl Passwd {
    /// Get a `passwd` entry from a raw [`libc::passwd`] pointer.
    ///
    /// # Errors
    ///
    /// [`PwdError::NullPtr`] - usually when an entry is non existent.
    #[must_use]
    pub fn from_raw(ptr: *mut libc::passwd) -> PwdResult<Self> {
        if ptr.is_null() {
            return Err(PwdError::NullPtr);
        }

        let raw = unsafe { *ptr };

        Ok(Self {
            name: cstr_to_string(raw.pw_name)?,
            password: match cstr_to_string(raw.pw_passwd) {
                Ok(p) => Some(p),
                Err(PwdError::NullPtr) => None,
                Err(e) => return Err(e),
            },
            uid: raw.pw_uid,
            gid: raw.pw_gid,
            gecos: match cstr_to_string(raw.pw_gecos) {
                Ok(p) => Some(p),
                Err(PwdError::NullPtr) => None,
                Err(e) => return Err(e),
            },
            directory: cstr_to_string(raw.pw_dir)?,
            shell: cstr_to_string(raw.pw_shell)?,
        })
    }

    /// Get a [`Passwd`] entry from a [`Uid`].
    #[must_use]
    pub fn from_uid(uid: Uid) -> PwdResult<Self> {
        Self::from_raw(unsafe { libc::getpwuid(uid) })
    }

    /// Get a [`Passwd`] entry from a user name.
    #[must_use]
    pub fn from_name(name: &str) -> PwdResult<Self> {
        Self::from_raw(unsafe { libc::getpwnam(CString::new(name)?.as_ptr()) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cstr_to_string() {
        let cstr = ::std::ffi::CString::new("test\x123").unwrap();
        assert_eq!(
            &cstr_to_string(cstr.as_ptr() as *mut c_char).unwrap()[..],
            "test\x123"
        )
    }

    #[test]
    #[should_panic]
    fn test_cstr_to_string_nul() {
        let cstr = ::std::ffi::CString::new("t\0est\x123").unwrap();
        assert_eq!(
            &cstr_to_string(cstr.as_ptr() as *mut c_char).unwrap()[..],
            "t\0est\x123"
        )
    }
}
