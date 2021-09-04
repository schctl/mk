//! Bindings for Unix's `libcrypt`.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::atomic::{AtomicBool, Ordering};

use mk_common::*;

/// Crypt function lock, since `crypt` isn't thread safe.
static CRYPT_LOCK: AtomicBool = AtomicBool::new(false);

/// Raw bindings to crypt headers.
pub mod ffi {
    #![allow(non_snake_case)]
    #![allow(non_camel_case_types)]
    #![allow(non_upper_case_globals)]

    include!(concat!(env!("OUT_DIR"), "/ffi.rs"));
}

/// Hash a passphrase.
///
/// # Errors
///
/// * [`FfiError::ResourceUnavailable`] - The function was called from multiple threads simultaneously.
/// * [`FfiError::InvalidPtr`] - Failed to hash the passphrase. This can occur if the system
///   does not support `crypt` at runtime as well.
///
/// See <https://manpages.debian.org/unstable/libcrypt-dev/crypt.3.en.html> for more.
pub fn crypt<'a>(phrase: &'a str, setting: &'a str) -> Result<&'a str, FfiError> {
    // Maybe check if passphrase starts with `*` and return `InvalidPtr`?
    //
    // From the linux man pages:
    //
    // > Upon error, crypt_r, crypt_rn, and crypt_ra write an invalid hashed passphrase to the output
    // > field of their data argument, and crypt writes an invalid hash to its static storage area.
    // > This string will be shorter than 13 characters, will begin with a `*', and will not compare
    // > equal to setting.

    if CRYPT_LOCK.load(Ordering::SeqCst) {
        return Err(FfiError::InvalidPtr);
    }

    CRYPT_LOCK.store(true, Ordering::SeqCst);

    let phrase = CString::new(phrase)?;
    let setting = CString::new(setting)?;

    let encrypted = unsafe {
        ffi::crypt(
            phrase.as_ptr() as *const c_char,
            setting.as_ptr() as *const c_char,
        )
    };

    CRYPT_LOCK.store(false, Ordering::SeqCst);

    if encrypted.is_null() {
        return Err(FfiError::InvalidPtr);
    }

    Ok(unsafe { CStr::from_ptr(encrypted).to_str()? })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_STR: &str = "Hello";
    const TEST_SET: &str = "$6$0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    const TEST_RES: &str = "$6$0000000000000000$rYZz1UNjyOm674sxVJ.lQ0fLFXMCBRjt.yZXhYx5S2PNgGBXrK9wXLY1ZZ0PhfM8be3g/lCW7TvKtzapAO4Lt/";

    #[test]
    fn test_crypt() {
        // Don't want these to run in parallel

        // First check to see if the hash works
        assert_eq!(crypt(TEST_STR, TEST_SET).unwrap(), TEST_RES);
        assert_eq!(crypt(TEST_STR, TEST_RES).unwrap(), TEST_RES);

        // Test from multiple threads
        test_concurrent_crypt();
    }

    /// When calling `crypt` from multiple threads at the same time, the output must always be either
    // an [`Ffi::InvalidPtr`] - indicating that another thread has called `crypt` first,
    /// or the correct hash, but never an invalid hash.
    fn test_concurrent_crypt() {
        const RUNS: usize = 1_000;
        const THREADS: usize = 4;

        fn run() {
            for _ in 0..RUNS {
                match crypt(TEST_STR, TEST_SET) {
                    Ok(TEST_RES) => {}
                    Err(FfiError::InvalidPtr) => {}
                    _ => panic!("Invalid result for crypt."),
                }
            }
        }

        let mut threads = Vec::with_capacity(THREADS);

        for _ in 0..THREADS {
            threads.push(::std::thread::spawn(|| run()))
        }

        for t in threads {
            t.join().unwrap();
        }
    }
}
