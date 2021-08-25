//! Bindings for Unix's `libcrypt`.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub mod ffi;

/// Utility wrapper around the [`ffi::crypt`] function.
pub fn crypt<'a>(phrase: &'a str, setting: &'a str) -> &'a str {
    unsafe {
        let phrase = CString::new(phrase).unwrap();
        let setting = CString::new(setting).unwrap();

        let encrypted = ffi::crypt(
            phrase.as_ptr() as *const c_char,
            setting.as_ptr() as *const c_char,
        );
        CStr::from_ptr(encrypted).to_str().unwrap()
    }
}
