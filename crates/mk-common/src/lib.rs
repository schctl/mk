//! A collection of commonly used APIs throughout all the `mk` crates.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::CStr;
use std::io;
use std::os::raw::c_char;
use std::sync::atomic::AtomicBool;

/// A global lock to check whether a non-safe ffi function is in use.
pub type GlobalFunctionLock = AtomicBool;

/// Helper macro to wrap an expression within a global function lock.
///
/// This is a commonly used pattern throughout the `mk` crates to wrap non-thread safe functions
/// and prevent thread races. If the given [`GlobalFunctionLock`] `$lock` is set to true, this
/// returns an [`Err`] containing `$if_lock`. Otherwise, it sets the lock to true, evaluates the
/// expression, resets the lock, and returns an [`Ok`] variant containing the evaluated expression.
#[macro_export]
macro_rules! function_lock {
    ($lock:expr, $val:expr, $if_lock:expr) => {{
        use std::sync::atomic::Ordering as __fn_lock_ordering;

        if $lock.load(__fn_lock_ordering::SeqCst) {
            Err($if_lock)
        } else {
            $lock.store(true, __fn_lock_ordering::SeqCst);
            let val = $val;
            $lock.store(false, __fn_lock_ordering::SeqCst);
            Ok(val)
        }
    }};
}

/// Wraps an [`io::Error`] in an [`Err`] variant. Intended to be a little neater to read.
///
/// - `$kind` corresponds to the [`io::ErrorKind`] of the error to create.
/// - `$val` corresponds to the payload contained within the error.
#[macro_export]
macro_rules! io_err {
    ($kind:ident, $val:expr) => {
        Err(::std::io::Error::new(::std::io::ErrorKind::$kind, $val).into())
    };

    ($kind:ident) => {
        Err(::std::io::Error::new(::std::io::ErrorKind::$kind, stringify!($kind)).into())
    };
}

/// Return with an [`Err`] variant containing an [`std::io::Error`].
#[macro_export]
macro_rules! io_bail {
    ($kind:ident, $val:expr) => {
        return $crate::io_err!($kind, $val)
    };

    ($kind:ident) => {
        return $crate::io_err!($kind)
    };
}

/// Run a generic expressionm

/// Wrapper around [`CStr`] to [`String`] conversion, converting errors to [`std::io::Error`].
pub unsafe fn cstr_to_string(ptr: *mut c_char) -> io::Result<String> {
    if ptr.is_null() {
        io_bail!(InvalidData, "null pointer");
    }

    match CStr::from_ptr(ptr).to_str() {
        Ok(s) => Ok(s.to_string()),
        Err(_) => io_err!(InvalidData, "invalid utf-8"),
    }
}

/// Get the `$PATH` variable from the environment.
///
/// Returns a fallback string if `$PATH` is not available.
#[must_use]
pub fn get_path() -> String {
    std::env::vars().find(|p| p.0 == "PATH").map_or(
        String::from("/usr/local/sbin:/usr/local/bin:/usr/bin"),
        |p| p.1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cstr_to_string() {
        let cstr = ::std::ffi::CString::new("test\x123").unwrap();
        assert_eq!(
            &unsafe { cstr_to_string(cstr.as_ptr() as *mut c_char).unwrap() }[..],
            "test\x123"
        )
    }
}
