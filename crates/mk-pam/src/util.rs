//! Commonly used functions.

use std::ffi::CStr;
use std::os::raw::c_char;

use mk_common::errors::FfiError;

use crate::errors::*;

/// Utility to convert from a C *[`c_char`] to a Rust [`String`] safely.
pub unsafe fn cstr_to_string(ptr: *const c_char) -> PamResult<String> {
    if ptr.is_null() {
        return Err(FfiError::InvalidPtr.into());
    }
    Ok(CStr::from_ptr(ptr).to_str()?.to_string())
}
