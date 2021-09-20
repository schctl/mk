//! A collection of commonly used APIs throughout all the `mk` crates.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::CStr;
use std::io::{Error, ErrorKind, Result};
use std::os::raw::c_char;

pub type Uid = libc::uid_t;
pub type Gid = libc::gid_t;
pub type Pid = libc::pid_t;

/// Wrapper around `*mut `[`c_char`] to [`String`] conversion, converting errors to [`io::Error`].
///
/// # Errors
///
/// - [`Error`] of kind [`ErrorKind::InvalidInput`] if `ptr` is null.
/// - [`Error`] of kind [`ErrorKind::InvalidData`] containing a [`Utf8Error`] if the string does
/// not contain valid utf-8 data.
///
/// # Safety
///
/// This behaves the same way as [`CStr::from_ptr`].
///
/// [`io::Error`]: Error
/// [`Utf8Error`]: std::str::Utf8Error
#[inline]
pub unsafe fn chars_to_string(ptr: *mut c_char) -> Result<String> {
    if ptr.is_null() {
        return Err(Error::new(ErrorKind::InvalidInput, "null pointer"));
    }

    match CStr::from_ptr(ptr).to_str() {
        Ok(s) => Ok(s.to_string()),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
    }
}
