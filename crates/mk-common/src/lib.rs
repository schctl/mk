//! A collection of commonly used APIs throughout all the `mk` crates.

use std::ffi::CStr;
use std::io::{Error, ErrorKind, Result};
use std::os::raw::c_char;
use std::time::Duration;

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
        Ok(s) => Ok(s.to_owned()),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
    }
}

/// Durations represented as seconds.
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationResolution {
    Seconds = 1,
    Minutes = 60,
    Days = 86_400,
}

/// Wrapper around serializing an [`Option`]`<`[`Duration`]`>` to an integer.
///
/// The resultant integer is `-1` if the duration is [`None`], else it is the duration represented
/// by the number of a given resolution. For egs, if a duration of 600s if given, with the resolution
/// set to [`DurationResolution::Minutes`], the resulting integer is `10`. This pattern is commonly used
/// in the `mk-` crates, so is defined as a utility here.
#[must_use]
#[inline]
pub fn ser_duration(dur: &Option<Duration>, res: DurationResolution) -> i64 {
    match dur {
        Some(d) => (d.as_secs() / res as u64) as i64,
        None => -1,
    }
}

/// Wrapper around deserializing an [`Option`]`<`[`Duration`]`>` from an integer.
///
/// This is the inverse of [`ser_duration`].
#[must_use]
#[inline]
pub fn de_duration(val: i64, res: DurationResolution) -> Option<Duration> {
    if val < 0 {
        None
    } else {
        Some(Duration::from_secs(val as u64 * res as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ser_duration;

    #[test]
    fn test_de_ser_duration() {
        let day = Duration::from_secs(DurationResolution::Days as u64);
        assert_eq!(ser_duration(&Some(day), DurationResolution::Days), 1);
        assert_eq!(de_duration(1, DurationResolution::Days), Some(day));
    }
}
