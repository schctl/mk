//! Bindings for Unix's `libcrypt`.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::ffi::{CStr, CString, NulError};
use std::os::raw::c_char;
use std::str::Utf8Error;

pub mod ffi;

/// Rust String to C String conversion errors.
#[derive(Debug, Clone)]
pub enum StrError {
    /// Interior nul byte was found.
    NulError(NulError),
    /// Invalid utf-8 when interpreting sequence of bytes as utf-8.
    Utf8Error(Utf8Error),
}

impl From<NulError> for StrError {
    fn from(e: NulError) -> Self {
        Self::NulError(e)
    }
}

impl From<Utf8Error> for StrError {
    fn from(e: Utf8Error) -> Self {
        Self::Utf8Error(e)
    }
}

/// Utility wrapper around the [`ffi::crypt`] function.
pub fn crypt<'a>(phrase: &'a str, setting: &'a str) -> Result<&'a str, StrError> {
    unsafe {
        let phrase = CString::new(phrase)?;
        let setting = CString::new(setting)?;

        let encrypted = ffi::crypt(
            phrase.as_ptr() as *const c_char,
            setting.as_ptr() as *const c_char,
        );
        Ok(CStr::from_ptr(encrypted).to_str()?)
    }
}
