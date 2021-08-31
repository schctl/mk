//! Utilities.

use std::ffi::CStr;

use libc::c_char;

use crate::errors::*;

/// Convert from a C *[`c_char`] to a Rust [`String`] safely.
pub fn cstr_to_string(ptr: *mut c_char) -> Result<String, FfiError> {
    if ptr.is_null() {
        return Err(FfiError::InvalidPtr);
    }
    Ok(unsafe { CStr::from_ptr(ptr) }.to_str()?.to_string())
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
