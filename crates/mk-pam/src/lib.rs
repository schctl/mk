//! Interface to PAM (Pluggable Authentication Modules).

#![allow(unused)]

/// Raw bindings to PAM headers.
pub mod ffi {
    #![allow(non_snake_case)]
    #![allow(non_camel_case_types)]
    #![allow(non_upper_case_globals)]

    include!(concat!(env!("OUT_DIR"), "/ffi.rs"));
}

use std::convert::TryFrom;

mod errors;
pub use errors::*;

pub mod conv;

use libc::c_int;

/// Safe interface to [`ffi::pam_start`].
///
/// This is the first of the Linux-PAM functions that must be called by an application.
/// It initializes the interface and reads the system configuration file, `/etc/pam.conf`.
/// Following a successful return, the contents of *pamh is a handle that provides continuity for
/// successive calls to the Linux-PAM library. The arguments expected by pam_start are as follows:
/// the service_name of the program, the username of the individual to be authenticated, a pointer
/// to an application-supplied pam_conv structure and a pointer to a pam_handle_t pointer.
pub fn start(
    service_name: &str,
    user_name: &str,
    conversation: conv::Conversation,
) -> PamResult<()> {
    // ffi::pam_start(
    //    service_name: *const ::std::os::raw::c_char,
    //    user: *const ::std::os::raw::c_char,
    //    pam_conversation: *const pam_conv,
    //    pamh: *mut *mut pam_handle_t
    // ) -> ::std::os::raw::c_int

    unimplemented!()
}
