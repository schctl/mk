//! Helpers for prompting the user.

use colored::*;
use std::os::unix::io::AsRawFd;

use crate::prelude::*;

// TODO:
// This is temporary, make this a structure.
// We should provide methods to format the prompt and configure options,
// somewhat similar to how a logger might be setup.
//
// See <https://docs.rs/fern/0.6.0/fern/>

/// Prompt user with a message on `stdout`, and read string from `stdin`.
///
/// If `stdout` is a terminal, the message is formatted in this way:
/// ```
///     [{SERVICE_NAME}][{user}] {msg}
/// ```
/// Otherwise, it is not formatted.
///
/// # Arguments
///
/// * `user` - Identification for the user.
/// * `msg` - Message to prompt user with.
/// * `is_password` - If this is set to true, a string is read from stdin as a password.
/// * `is_colored` - If this is set to true, the prompt is formatted with colors.
pub fn prompt(user: &str, msg: &str, is_password: bool, is_colored: bool) -> MkResult<String> {
    // This is a lot of spaghetti but we'll fix it later
    let prompt = if is_colored && unsafe { libc::isatty(std::io::stdout().as_raw_fd()) == 1 } {
        format!(
            "[{}][{}] {}",
            if is_colored { SERVICE_NAME.dimmed() } else { SERVICE_NAME.normal() },
            if is_colored { user.bold().italic() } else { user.normal() },
            msg
        )
    } else {
        msg.to_string()
    };

    if is_password {
        Ok(rpassword::prompt_password_stdout(&prompt[..])?)
    } else {
        print!("{}", prompt);
        let mut res = String::new();
        std::io::stdin().read_line(&mut res)?;
        Ok(res)
    }
}
