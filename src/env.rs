/// Environment setup utilities for running a command.
use std::ffi::CString;

/// Get environment variables in a list of `key=value` pairs.
pub fn create_env(user: pwd::Passwd) -> Vec<CString> {
    // Very rudimentary environment creation.

    let mut vars = vec![];

    // vars.push(CString::new(format!("{}={}", "USER", user.name)).unwrap());
    // vars.push(CString::new(format!("{}={}", "HOME", user.dir)).unwrap());
    // vars.push(CString::new(format!("{}={}", "SHELL", user.shell)).unwrap());
    vars.push(CString::new("").unwrap());

    vars
}

/// Get argument list.
pub fn get_args() -> Vec<CString> {
    let mut args = std::env::args();
    let _ = args.next().unwrap();
    args.map(|v| CString::new(v).unwrap()).collect()
}
