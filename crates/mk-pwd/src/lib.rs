//! Rust interface to POSIX's `pwd.h`.
//!
//! See also [`pwd.h(0p)`](https://man7.org/linux/man-pages/man0/pwd.h.0p.html).

use std::ffi::CString;
use std::io;
use std::sync::Mutex;

use mk_common::*;

lazy_static::lazy_static! {
    /// `getpwnam` is not thread safe. This is a safe guard against thread races.
    /// See <https://man7.org/linux/man-pages/man3/getpwnam.3p.html#DESCRIPTION>.
    pub(crate) static ref PWNAME_LOCK: Mutex<()> = Mutex::new(());
}

/// A single entry in the password database.
///
/// See [`passwd(5)`](https://man7.org/linux/man-pages/man5/passwd.5.html) for more.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Passwd {
    /// User's login name.
    pub name: String,
    /// This is either the encrypted user password, an asterisk (*), or the letter 'x'.
    pub password: Option<String>,
    /// User's unique ID.
    pub uid: Uid,
    /// User's numeric primary group ID.
    pub gid: Gid,
    /// Comment field, ssed for informational purposes.
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
    /// This function can fail if any of the information contains invalid utf-8. See [`chars_to_string`].
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid pointer to a [`libc::passwd`].
    pub unsafe fn from_raw(ptr: *mut libc::passwd) -> io::Result<Self> {
        let raw = *ptr;

        Ok(Self {
            name: chars_to_string(raw.pw_name)?,
            password: match chars_to_string(raw.pw_passwd) {
                // Set to nullptr if user doesn't have a password
                Ok(p) => Some(p),
                Err(_) => None,
            },
            uid: raw.pw_uid,
            gid: raw.pw_gid,
            gecos: match chars_to_string(raw.pw_gecos) {
                Ok(p) => Some(p),
                Err(_) => None,
            },
            directory: chars_to_string(raw.pw_dir)?,
            shell: chars_to_string(raw.pw_shell)?,
        })
    }

    /// Get a [`Passwd`] entry from a [`Uid`].
    ///
    /// # Errors
    ///
    /// - [`io::Error`] if a user was not found or if an error occurred while processing.
    pub fn from_uid(uid: Uid) -> io::Result<Self> {
        // SAFETY: `getpwnam` and `getpwuid` return a null pointer if an entry is not available, or if some
        // other error occurs during processing. We handle this with an early exit. Thread races
        // are checked using the global `PWNAME_LOCK`.
        unsafe {
            let _lock = PWNAME_LOCK.lock().unwrap();

            let ptr = libc::getpwuid(uid);

            if ptr.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("could not find user with UID {}", uid),
                ));
            }

            Self::from_raw(ptr)
        }
    }

    /// Get a [`Passwd`] entry from a user name.
    ///
    /// # Errors
    ///
    /// - [`io::Error`] if a user was not found or if an error occurred while processing.
    pub fn from_name(name: &str) -> io::Result<Self> {
        // SAFETY: `getpwnam` and `getpwuid` return a null pointer if an entry is not available, or if some
        // other error occurs during processing. We handle this with an early exit. Thread races
        // are checked using the global `PWNAME_LOCK`.
        unsafe {
            let _lock = PWNAME_LOCK.lock().unwrap();

            let c_name = CString::new(name)?;
            let ptr = libc::getpwnam(c_name.as_ptr());

            if ptr.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("could not find user {}", name),
                ));
            }

            Self::from_raw(ptr)
        }
    }
}
