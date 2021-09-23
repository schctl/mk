//! Random utility functions.

use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

use mk_common::*;

/// Get the `PATH` variable from the environment.
///
/// Returns a fallback string if it is not available.
#[must_use]
#[inline]
pub fn get_path() -> String {
    std::env::vars().find(|p| p.0 == "PATH").map_or(
        String::from("/usr/local/sbin:/usr/local/bin:/usr/bin"),
        |p| p.1,
    )
}

/// Change a given file's mode.
pub fn set_mode<P: AsRef<Path>>(path: P, mode: u32) -> io::Result<()> {
    let mut perms = fs::metadata(path.as_ref())?.permissions();
    perms.set_mode(mode);
    fs::set_permissions(path, perms)?;
    Ok(())
}

/// Read a line from `/dev/tty`.
pub fn readln_from_tty() -> io::Result<String> {
    let mut input = String::new();
    let tty = File::open("/dev/tty")?;
    let mut reader = BufReader::new(tty);
    reader.read_line(&mut input)?;
    Ok(input)
}

pub mod timeout_serializer {
    use super::*;

    use serde::{Deserialize, Serialize};

    pub fn serialize<S: serde::Serializer>(
        dur: &Option<Duration>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ser_duration(dur, DurationResolution::Minutes).serialize(serializer)
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Duration>, D::Error> {
        let val = i64::deserialize(deserializer)?;

        Ok(de_duration(val, DurationResolution::Minutes))
    }
}

/// Read a password from the tty.
macro_rules! password_from_tty {
    ($($arg:tt)*) => {
        ::rpassword::read_password_from_tty(Some(&format!($($arg)*)[..]))
    };
}

/// Prompt text and read a line from `/dev/tty`.
macro_rules! prompt_from_tty {
    ($($arg:tt)*) => {
        {
            print!($($arg)*);
            $crate::utils::readln_from_tty()
        }
    };
}
