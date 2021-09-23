//! Interface to the system provided shadow routines.
//!
//! See also [`shadow(3)`](https://www.man7.org/linux/man-pages/man3/shadow.3.html).

use std::ffi::CString;
use std::io;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use mk_common::{chars_to_string, de_duration, DurationResolution};

lazy_static::lazy_static! {
    /// Shadow routines are not thread safe.
    /// See <https://www.man7.org/linux/man-pages/man3/getspnam.3.html#ATTRIBUTES>.
    static ref LOCK: Mutex<()> = Mutex::new(());
    static ref ENT_LOCK: Mutex<()> = Mutex::new(());
}

/// A single entry in the shadow file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spwd {
    /// User's login name.
    pub name: String,
    /// Encrypted password.
    pub password: String,
    /// Date of last password change.
    pub date_change: Option<SystemTime>,
    /// The duration that must pass before the user's password can be changed.
    pub password_age_min: Option<Duration>,
    /// The duration after which the user's password must be changed.
    pub password_age_max: Option<Duration>,
    /// The duration before password expiry, in which the user will be warned to change their password.
    pub warn_duration: Option<Duration>,
    /// The duration after password expiry, after which the user's account will be disabled.
    pub inactive_duration: Option<Duration>,
    /// Date when the user's account expired.
    pub expiry: Option<SystemTime>,
}

impl Spwd {
    /// Create a `Spwd` struct from a raw [`libc::spwd`] pointer, allocating new resources for it.
    ///
    /// # Errors
    ///
    /// This function can fail if any of the information contains invalid utf-8. See [`chars_to_string`].
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid pointer to a [`libc::spwd`].
    pub unsafe fn from_raw(ptr: *mut libc::spwd) -> io::Result<Self> {
        let raw = *ptr;

        Ok(Self {
            name: chars_to_string(raw.sp_namp)?,
            password: chars_to_string(raw.sp_pwdp)?,
            date_change: de_duration(raw.sp_lstchg, DurationResolution::Days)
                .map(|d| SystemTime::UNIX_EPOCH + d),
            password_age_min: de_duration(raw.sp_min, DurationResolution::Days),
            password_age_max: de_duration(raw.sp_max, DurationResolution::Days),
            warn_duration: de_duration(raw.sp_warn, DurationResolution::Days),
            inactive_duration: de_duration(raw.sp_inact, DurationResolution::Days),
            expiry: de_duration(raw.sp_expire, DurationResolution::Days)
                .map(|d| SystemTime::UNIX_EPOCH + d),
        })
    }

    /// Get a [`Spwd`] entry from a user name.
    ///
    /// # Errors
    ///
    /// - [`io::Error`] if a user was not found or if an error occurred while processing.
    ///
    /// # Panics
    ///
    /// Panics if a previous call to this function from another thread panicked.
    pub fn from_name(name: &str) -> io::Result<Self> {
        // SAFETY: shadow routines return a null pointer if an entry is not available, or if some
        // other error occurs during processing. We handle this with an early exit. Thread races
        // are checked using the global `SPNAME_LOCK`.
        unsafe {
            let _lock = LOCK.lock().unwrap();

            let c_name = CString::new(name)?;
            let ptr = libc::getspnam(c_name.as_ptr());

            if ptr.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "could not read shadow file",
                ));
            }

            Self::from_raw(ptr)
        }
    }
}

/// An iterator over entries in the shadow password database.
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
    /// Construct a new iterator over the shadow password database entries.
    #[must_use]
    pub fn new() -> Self {
        Self { index: 0 }
    }
}

impl Iterator for Entries {
    type Item = io::Result<Spwd>;

    /// NOTE: this function rewinds to the beginning of the shadow password database each time it is called.
    /// This is bad for performance but ensures that the entries returned are correct.
    fn next(&mut self) -> Option<Self::Item> {
        // This whole thing is for thread safety.
        // Two entries being iterated over concurrently will interfere with the stream position of the other.
        // By rewinding and reiterating over all the elements, we ensure that no entries get skipped.

        let lock = ENT_LOCK.lock().unwrap();

        unsafe {
            libc::setspent();

            let mut ptr = std::ptr::null_mut();

            for _ in 0..=self.index {
                ptr = libc::getspent();

                if ptr.is_null() {
                    return None;
                }
            }

            libc::endspent();

            self.index += 1;
            std::mem::drop(lock);
            Some(Spwd::from_raw(ptr))
        }
    }
}
