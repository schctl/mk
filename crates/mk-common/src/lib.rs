//! A collection of commonly used APIs throughout all the `mk` crates.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::CStr;
use std::io;
use std::os::raw::c_char;

/// Helper macro to get an [`Err`] variant from an [`std::io::Error`].
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
        return io_err!($kind, $val)
    };

    ($kind:ident) => {
        return io_err!($kind)
    };
}

/// Convert from a `C` [`*c_char`](c_char) to a Rust [`String`] safely.
pub fn cstr_to_string(ptr: *mut c_char) -> io::Result<String> {
    if ptr.is_null() {
        io_bail!(InvalidData, "null pointer");
    }

    match unsafe { CStr::from_ptr(ptr) }.to_str() {
        Ok(s) => Ok(s.to_string()),
        Err(_) => io_err!(InvalidData, "invalid utf-8"),
    }
}

/// Get the `$PATH` variable from the environment.
///
/// Returns a fallback string if `$PATH` is not available.
pub fn get_path() -> String {
    if let Some(p) = std::env::vars().find(|p| p.0 == "PATH") {
        p.1
    } else {
        String::from("/usr/local/sbin:/usr/local/bin:/usr/bin")
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
}
