//! Helpers for prompting the user.

// TODO:
// This is temporary, make this a structure.
// We should provide methods to format the prompt and configure options,
// somewhat similar to how a logger might be setup.
//
// See <https://docs.rs/fern/0.6.0/fern/>

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
            let __prompt_msg = format!(
                "[{}] {}",
                module_path!(),
                format_args!($($arg)*)
            );

            if $pwd {
                rpassword::prompt_password_stdout(&__prompt_msg[..])
            } else {
                print!("{}", __prompt_msg);
                let mut res = String::new();
                match std::io::stdin().read_line(&mut res) {
                    Ok(_) => Ok(res),
                    Err(e) => Err(e)
                }
            }
        }
    };
}
