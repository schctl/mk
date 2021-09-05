//! Utilities.

use std::ffi::CStr;
use std::io;
use std::os::raw::c_char;

use crate::nullptr_bail;

/// Convert from a `C` [`*c_char`](c_char) to a Rust [`String`] safely.
pub fn cstr_to_string(ptr: *mut c_char) -> io::Result<String> {
    if ptr.is_null() {
        nullptr_bail!();
    }

    match unsafe { CStr::from_ptr(ptr) }.to_str() {
        Ok(s) => Ok(s.to_string()),
        Err(_) => Err(io::Error::new(io::ErrorKind::InvalidData, "invalid utf-8")),
    }
}

/// Get the `$PATH` variable from the environment.
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
