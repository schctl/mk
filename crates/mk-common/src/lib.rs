//! A collection of commonly used APIs throughout all the `mk` crates.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::CStr;
use std::io;
use std::os::raw::c_char;
use std::sync::atomic::{AtomicBool, Ordering};

/// A lock to indicate that it is not safe to use a resource now.
pub type ResourceLock = AtomicBool;

pub type Uid = libc::uid_t;
pub type Gid = libc::gid_t;
pub type Pid = libc::pid_t;

/// Helper function to wrap an execution within a global function lock.
///
/// This is a commonly used pattern throughout the `mk` crates to wrap ffi functions that are
/// not thread safe and prevent thread races. If the given [`ResourceLock`] `lock` is set to
/// true, this returns an [`Err`] containing `if_lock`. Otherwise, it sets the lock to true, evaluates
/// the expression, resets the lock, and returns an [`Ok`] containing the evaluated expression.
pub fn fn_lock<T, E, S: FnOnce() -> T, F: FnOnce() -> E>(
    lock: &ResourceLock,
    val: S,
    if_lock: F,
) -> Result<T, E> {
    if lock
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        let res = val();
        lock.store(false, Ordering::SeqCst);
        Ok(res)
    } else {
        Err(if_lock())
    }
}

/// Wrapper around [`CStr`] to [`String`] conversion, converting errors to [`std::io::Error`].
///
/// # Safety
///
/// This behaves the same way as [`CStr::from_ptr`].
pub unsafe fn cstr_to_string(ptr: *mut c_char) -> io::Result<String> {
    if ptr.is_null() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "null pointer"));
    }

    match CStr::from_ptr(ptr).to_str() {
        Ok(s) => Ok(s.to_string()),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

/// Get the real user ID of the calling process.
#[must_use]
#[inline]
pub fn get_uid() -> Uid {
    unsafe { libc::getuid() }
}

/// Get the effective user ID of the calling process.
#[must_use]
#[inline]
pub fn get_euid() -> Uid {
    unsafe { libc::geteuid() }
}

// Get the PID of the parent process.
#[must_use]
#[inline]
pub fn get_parent_pid() -> Pid {
    unsafe { libc::getppid() }
}
