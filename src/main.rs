//! `mk` is a tool to run unix commands as another user, and a family of crates. It is similar to
//! [`doas`](https://github.com/Duncaen/OpenDoas) or [`sudo`](https://github.com/sudo-project/sudo).
//!
//! # Feature flags
//!
//! | Flag | Description | Requires | Rust |
//! |------|-------------|----------|------|
//! | `pam` | Builds with authenticator support for [`PAM`](https://en.wikipedia.org/wiki/Pluggable_authentication_module) | A `PAM` implementation ([`Linux-PAM`](http://www.linux-pam.org/), [`OpenPAM`](https://www.openpam.org/)) | 1.56.0-nightly  |
//! | `shadow` | Builds with support for authentication using [`shadow-utils`](https://github.com/shadow-maint/shadow) | System provided `shadow.h` | 1.45+ |

#[macro_use]
pub mod util;

mod auth;
mod cli;
mod config;
mod errors;
mod options;
mod prelude;

pub use errors::*;

use config::Config;

fn main() {
    let mut app = cli::App::new(Config {}).unwrap();

    match app.run(cli::options_from_terminal().unwrap()) {
        Err(e) => eprintln!("{}", e),
        Ok(Some(e)) => ::std::process::exit(e),
        _ => {}
    };
}
