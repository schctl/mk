//! Random utility functions.

use std::ffi::CStr;

use libc::c_char;
use mk_pwd::Uid;

use crate::prelude::*;

/// Get the real user ID of the calling process.
pub fn get_uid() -> Uid {
    unsafe { libc::getuid() }
}

/// Get the effective user ID of the calling process.
pub fn get_euid() -> Uid {
    unsafe { libc::geteuid() }
}

/// Utility to convert a C *[`c_char`] to a Rust [`String`] safely.
pub fn cstr_to_string(ptr: *mut c_char) -> MkResult<String> {
    if ptr.is_null() {
        return Err(MkError::NullPtr);
    }
    Ok(unsafe { CStr::from_ptr(ptr) }.to_str()?.to_string())
}
