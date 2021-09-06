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

/// Prompt user with a message on `stdout`, and read string from `stdin`.
///
/// The message is formatted in this way:
/// ```
/// [{module_path}] {msg}
/// ```
///
/// If `$pwd` is true, a string is read as a password.
macro_rules! prompt {
    ($pwd:expr, $($arg:tt)*) => {
        {
            if $pwd {
                rpassword::prompt_password_stdout(&format!($($arg)*)[..])
            } else {
                print!($($arg)*);
                let mut res = String::new();
                match std::io::stdin().read_line(&mut res) {
                    Ok(_) => Ok(res),
                    Err(e) => Err(e)
                }
            }
        }
    };
}
