//! `mk` is a tool to run unix commands as another user, and a family of crates. It is similar to
//! [`doas`](https://github.com/Duncaen/OpenDoas) or [`sudo`](https://github.com/sudo-project/sudo).
//!
//! # Feature flags
//!
//! | Flag | Description | Requires | Rust |
//! |------|-------------|----------|------|
//! | `pam` | Builds with authenticator support for [`PAM`](https://en.wikipedia.org/wiki/Pluggable_authentication_module) | A `PAM` implementation ([`Linux-PAM`](http://www.linux-pam.org/), [`OpenPAM`](https://www.openpam.org/)) | 1.56.0-nightly  |
//! | `shadow` | Builds with support for authentication using [`shadow-utils`](https://github.com/shadow-maint/shadow) | System provided `shadow.h` | 1.54+ |

#[macro_use]
pub mod util;

pub mod auth;
pub mod cli;
pub mod config;
pub mod errors;
pub mod options;
pub mod prelude;

pub use errors::*;
