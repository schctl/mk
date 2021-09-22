//! Interface to POSIX's `pwd.h`.
//!
//! See also [`pwd.h(0p)`](https://man7.org/linux/man-pages/man0/pwd.h.0p.html).

use std::ffi::CString;
use std::io;
use std::sync::Mutex;

use mk_common::{chars_to_string, Gid, Uid};

lazy_static::lazy_static! {
    /// This is a safe guard against thread races.
    /// See <https://man7.org/linux/man-pages/man3/getpwnam.3p.html#DESCRIPTION>.
    static ref LOCK: Mutex<()> = Mutex::new(());
    static ref ENT_LOCK: Mutex<()> = Mutex::new(());
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
    /// Create a `Passwd` struct from a raw [`libc::passwd`] pointer, allocating new resources for it.
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
            // Set to nullptr if non-existent
            password: chars_to_string(raw.pw_passwd).ok(),
            uid: raw.pw_uid,
            gid: raw.pw_gid,
            // Set to nullptr if non-existent
            gecos: chars_to_string(raw.pw_gecos).ok(),
            directory: chars_to_string(raw.pw_dir)?,
            shell: chars_to_string(raw.pw_shell)?,
        })
    }

    /// Get a [`Passwd`] entry from a [`Uid`].
    ///
    /// # Errors
    ///
    /// - [`io::Error`] if a user was not found or if an error occurred while processing.
    ///
    /// # Panics
    ///
    /// Panics if a previous call to [`from_name`] or [`from_uid`] from another thread panicked.
    ///
    /// [`from_name`]: Self::from_name
    /// [`from_uid`]: Self::from_uid
    pub fn from_uid(uid: Uid) -> io::Result<Self> {
        // SAFETY: `getpwnam` and `getpwuid` return a null pointer if an entry is not available, or if some
        // other error occurs during processing. We handle this with an early exit. Thread races
        // are checked using the global `PWNAME_LOCK`.
        unsafe {
            let _lock = LOCK.lock().unwrap();

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
    ///
    /// # Panics
    ///
    /// Panics if a previous call to [`from_name`] or [`from_uid`] from another thread panicked.
    ///
    /// [`from_name`]: Self::from_name
    /// [`from_uid`]: Self::from_uid
    pub fn from_name(name: &str) -> io::Result<Self> {
        // SAFETY: `getpwnam` and `getpwuid` return a null pointer if an entry is not available, or if some
        // other error occurs during processing. We handle this with an early exit. Thread races
        // are checked using the global `PWNAME_LOCK`.
        unsafe {
            let _lock = LOCK.lock().unwrap();

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

/// An iterator over entries in the system password database.
pub struct Entries {
    /// Index
    index: usize,
}

impl Default for Entries {
    fn default() -> Self {
        Self::new()
    }
}

impl Entries {
    /// Construct a new iterator over the password database entries.
    #[must_use]
    pub fn new() -> Self {
        Self { index: 0 }
    }
}

impl Iterator for Entries {
    type Item = io::Result<Passwd>;

    /// NOTE: this function rewinds to the beginning of the password database each time it is called.
    /// This is bad for performance but ensures that the entries returned are correct.
    fn next(&mut self) -> Option<Self::Item> {
        // This whole thing is for thread safety.
        // Two entries being iterated over concurrently will interfere with the stream position of the other.
        // By rewinding and reiterating over all the elements, we ensure that no entries get skipped.

        let lock = ENT_LOCK.lock().unwrap();

        unsafe {
            libc::setpwent();

            let mut ptr = std::ptr::null_mut();

            for _ in 0..=self.index {
                ptr = libc::getpwent();

                if ptr.is_null() {
                    return None;
                }
            }

            libc::endpwent();

            self.index += 1;
            std::mem::drop(lock);
            Some(Passwd::from_raw(ptr))
        }
    }
}
