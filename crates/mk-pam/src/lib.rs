//! Raw interface to PAM (Pluggable Authentication Modules).
//!
//! This crate provides **experimental** safe interfaces to PAM as well.

#![feature(vec_into_raw_parts)]

use std::os::raw::{c_int, c_void};
use std::{convert::TryFrom, ffi::CString};

use mk_common::errors::FfiError;

pub mod conv;
pub mod errors;

mod prelude;
pub(crate) mod util;
use prelude::*;

/// Raw bindings to PAM headers.
pub mod ffi {
    #![allow(non_snake_case)]
    #![allow(non_camel_case_types)]
    #![allow(non_upper_case_globals)]

    include!(concat!(env!("OUT_DIR"), "/ffi.rs"));
}

// See http://uw714doc.sco.com/en/SEC_pam/pam_appl-3.html

/// Represents a PAM handle.
pub struct Handle {
    pub interior: *mut ffi::pam_handle,
}

impl Handle {
    /// Safe interface to [`ffi::pam_start`].
    ///
    /// This is the first of the PAM functions that must be called by an application.
    /// It initializes the interface and reads the system configuration file, `/etc/pam.conf`.
    /// Following a successful return, the contents of *pamh is a handle that provides continuity for
    /// successive calls to the PAM library.
    pub fn start(
        service_name: &str,
        user_name: &str,
        conversation: conv::Conversation,
    ) -> PamResult<Self> {
        let service_name = CString::new(service_name)?;
        let user_name = CString::new(user_name)?;

        let conv = {
            // Insert conversation into the global conversation map
            // with its id as the index in the map.
            let mut global_ptr_lock = conv::GLOBAL_CONV_PTRS.lock().unwrap();
            let index = global_ptr_lock.len();
            global_ptr_lock.insert(index as c_int, conversation);

            ffi::pam_conv {
                conv: Some(conv::__raw_pam_conv),
                // This is a very hack-y approach.
                // This is actually an invalid pointer, but in `__raw_pam_conv`
                // we treat the pointer's address as a number, and use that as an index.
                appdata_ptr: { index as *mut c_void },
            }
        };

        let mut pamh: *mut ffi::pam_handle = std::ptr::null_mut();

        let ret =
            unsafe { ffi::pam_start(service_name.as_ptr(), user_name.as_ptr(), &conv, &mut pamh) };

        if pamh.is_null() {
            return Err(FfiError::InvalidPtr.into());
        }

        match RawError::try_from(ret as i32) {
            Ok(e) => Err(e.into()),
            Err(_) => Ok(Self { interior: pamh }),
        }
    }
}
