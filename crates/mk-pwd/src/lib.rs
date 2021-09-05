//! Rust interface to Unix's `pwd.h`.

use std::ffi::CString;
use std::io;

use mk_common::*;

pub type Uid = libc::uid_t;
pub type Gid = libc::gid_t;

/// A single entry in `/etc/passwd`.
///
/// The `/etc/passwd` file is a text file that describes user login accounts for the system.
/// Each line of the file describes a single user, this struct is a representation of each entry.
///
/// See <https://linux.die.net/man/5/passwd> for more.
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
    /// [`FfiError::InvalidPtr`] - usually when an entry is non existent.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid pointer to a [`libc::passwd`].
    pub unsafe fn from_raw(ptr: *mut libc::passwd) -> io::Result<Self> {
        if ptr.is_null() {
            nullptr_bail!();
        }

        let raw = *ptr;

        Ok(Self {
            name: util::cstr_to_string(raw.pw_name)?,
            password: match util::cstr_to_string(raw.pw_passwd) {
                // Set to nullptr if user doesn't have a password
                Ok(p) => Some(p),
                Err(_) => None,
            },
            uid: raw.pw_uid,
            gid: raw.pw_gid,
            gecos: match util::cstr_to_string(raw.pw_gecos) {
                Ok(p) => Some(p),
                Err(_) => None,
            },
            directory: util::cstr_to_string(raw.pw_dir)?,
            shell: util::cstr_to_string(raw.pw_shell)?,
        })
    }

    /// Get a [`Passwd`] entry from a [`Uid`].
    pub fn from_uid(uid: Uid) -> io::Result<Self> {
        unsafe { Self::from_raw(libc::getpwuid(uid)) }
    }

    /// Get a [`Passwd`] entry from a user name.
    pub fn from_name(name: &str) -> io::Result<Self> {
        unsafe { Self::from_raw(libc::getpwnam(CString::new(name)?.as_ptr())) }
    }
}
