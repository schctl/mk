//! Interface to the system provided shadow routines.
//!
//! See also [`shadow(3)`](https://www.man7.org/linux/man-pages/man3/shadow.3.html).

use std::ffi::{CStr, CString};
use std::io;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use mk_common::{chars_to_string, de_duration, DurationResolution};

lazy_static::lazy_static! {
    /// Shadow routines are not thread safe.
    /// See <https://www.man7.org/linux/man-pages/man3/getspnam.3.html#ATTRIBUTES>.
    static ref LOCK: Mutex<()> = Mutex::new(());
}

/// A single entry in the shadow file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spwd {
    /// User's login name.
    pub name: String,
    /// Encrypted password.
    pub password: CString,
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
    /// Date when the user's account expires.
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
            password: CStr::from_ptr(raw.sp_pwdp).to_owned(),
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
        let lock = LOCK.lock().unwrap();

        // SAFETY: shadow routines return a null pointer if an entry is not available, or if some
        // other error occurs during processing. We handle this with an early exit.
        unsafe {
            let c_name = CString::new(name)?;
            let ptr = libc::getspnam(c_name.as_ptr());

            if ptr.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "could not read shadow file",
                ));
            }

            drop(lock);
            Self::from_raw(ptr)
        }
    }
}
