//! Utilities.

use std::ffi::CStr;
use std::os::raw::c_char;

use crate::errors::*;

/// Convert from a `C` [`*c_char`](c_char) to a Rust [`String`] safely.
pub fn cstr_to_string(ptr: *mut c_char) -> Result<String, FfiError> {
    if ptr.is_null() {
        return Err(FfiError::InvalidPtr);
    }
    Ok(unsafe { CStr::from_ptr(ptr) }.to_str()?.to_string())
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

    #[test]
    fn test_cstr_from_nullptr() {
        match cstr_to_string(::std::ptr::null_mut()) {
            Ok(_) => panic!("Null pointer conversion somehow succeeded?"),
            Err(e) => match e {
                FfiError::InvalidPtr => {}
                _ => panic!("Null pointer conversion error is not invalid ptr."),
            },
        }
    }
}
