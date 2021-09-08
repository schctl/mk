//! `mk` is a tool to run unix commands as another user, and a family of crates. It is similar to
//! [`doas`](https://github.com/Duncaen/OpenDoas) or [`sudo`](https://github.com/sudo-project/sudo).

#[macro_use]
pub mod util;

pub mod auth;
pub mod cli;
pub mod config;
pub mod errors;
pub mod options;
pub mod prelude;

pub use errors::*;
