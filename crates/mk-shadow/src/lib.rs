//! Rust interface to the system provided shadow routines.
//!
//! ## Potentially useful links
//!
//! - [`shadow(3)`](https://www.man7.org/linux/man-pages/man3/shadow.3.html)
//! - [`getspnam(3)`](https://www.man7.org/linux/man-pages/man3/getspnam.3.html)

use std::ffi::CString;
use std::io;
use std::time;

use mk_common::*;

/// Shadow routines are not thread safe. This is a safe guard against thread races.
/// See <https://www.man7.org/linux/man-pages/man3/getspnam.3.html#ATTRIBUTES>.
static SPNAME_LOCK: ResourceLock = ResourceLock::new(false);

/// A single entry in the shadow file.
#[derive(Debug, Clone)]
pub struct Spwd {
    /// User's login name.
    pub name: String,
    /// Encrypted password.
    pub password: String,
    /// Date of last password change.
    pub date_change: Option<std::time::SystemTime>,
    /// The duration that must pass before the user's password can be changed.
    pub password_age_min: Option<time::Duration>,
    /// The duration after which the user's password must be changed.
    pub password_age_max: Option<time::Duration>,
    /// The duration before password expiry, in which the user will be warned to change their password.
    pub password_warn_duration: Option<time::Duration>,
    /// The duration after password expiry, after which the user's account will be disabled.
    pub password_inactive_duration: Option<time::Duration>,
    /// Date when the user's account expired.
    pub expiry: Option<time::SystemTime>,
    /// Reserved field.
    pub flag: u64,
}

impl Spwd {
    /// Get a [`Spwd`] struct from a raw [`libc::spwd`] pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid pointer to a [`libc::spwd`].
    pub unsafe fn from_raw(ptr: *mut libc::spwd) -> io::Result<Self> {
        let raw = *ptr;

        Ok(Self {
            name: cstr_to_string(raw.sp_namp)?,
            password: cstr_to_string(raw.sp_pwdp)?,
            date_change: if raw.sp_lstchg >= 0 {
                Some(time::UNIX_EPOCH + time::Duration::from_secs((raw.sp_lstchg as u64) * 86_400))
            } else {
                None
            },
            password_age_min: if raw.sp_min >= 0 {
                Some(time::Duration::from_secs((raw.sp_min as u64) * 86_400))
            } else {
                None
            },
            password_age_max: if raw.sp_max >= 0 {
                Some(time::Duration::from_secs((raw.sp_max as u64) * 86_400))
            } else {
                None
            },
            password_warn_duration: if raw.sp_warn >= 0 {
                Some(time::Duration::from_secs((raw.sp_warn as u64) * 86_400))
            } else {
                None
            },
            password_inactive_duration: if raw.sp_inact >= 0 {
                Some(time::Duration::from_secs((raw.sp_inact as u64) * 86_400))
            } else {
                None
            },
            expiry: if raw.sp_expire >= 0 {
                Some(time::UNIX_EPOCH + time::Duration::from_secs((raw.sp_expire as u64) * 86_400))
            } else {
                None
            },
            flag: raw.sp_flag as u64,
        })
    }

    /// Get a [`Spwd`] entry from a user name.
    ///
    /// # Errors
    ///
    /// - [`io::Error`] if a user was not found or if an error occurred while processing.
    pub fn from_name(name: &str) -> io::Result<Self> {
        // SAFETY: shadow routines return a null pointer if an entry is not available, or if some
        // other error occurs during processing. We handle this with an early exit. Thread races
        // are checked using the global `SPNAME_LOCK`.
        unsafe {
            let cname = CString::new(name)?;

            let ptr = fn_lock(
                &SPNAME_LOCK,
                || libc::getspnam(cname.as_ptr()),
                || io::Error::new(io::ErrorKind::Interrupted, "thread locked"),
            )?;

            if ptr.is_null() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "null pointer"));
            }

            Self::from_raw(ptr)
        }
    }

    /// Get a [`Spwd`] entry from a [`mk_pwd::Passwd`] entry.
    ///
    /// # Errors
    ///
    /// - [`io::Error`] if a user was not found or if an error occurred while processing.
    pub fn from_passwd(pwd: &mk_pwd::Passwd) -> io::Result<Self> {
        Self::from_name(&pwd.name[..])
    }
}
