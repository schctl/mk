//! Random utility functions.

#![allow(unused_macros)]

use std::fs::File;
use std::io::{self, BufRead, BufReader};

use mk_pwd::Uid;

/// Get the real user ID of the calling process.
pub fn get_uid() -> Uid {
    unsafe { libc::getuid() }
}

/// Get the effective user ID of the calling process.
pub fn get_euid() -> Uid {
    unsafe { libc::geteuid() }
}

/// Get the `PATH` variable from the environment.
///
/// Returns a fallback string if it is not available.
#[must_use]
pub fn get_path() -> String {
    std::env::vars().find(|p| p.0 == "PATH").map_or(
        String::from("/usr/local/sbin:/usr/local/bin:/usr/bin"),
        |p| p.1,
    )
}

/// Read a line from `/dev/tty`.
pub fn readln_from_tty() -> io::Result<String> {
    let mut input = String::new();
    let tty = File::open("/dev/tty")?;
    let mut reader = BufReader::new(tty);
    reader.read_line(&mut input)?;
    Ok(input)
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
            $crate::util::readln_from_tty()
        }
    };
}
