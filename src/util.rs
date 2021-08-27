//! Random utility functions.

use mk_pwd::Uid;

/// Get the real user ID of the calling process.
pub fn get_uid() -> Uid {
    unsafe { libc::getuid() }
}

/// Get the effective user ID of the calling process.
pub fn get_euid() -> Uid {
    unsafe { libc::geteuid() }
}
