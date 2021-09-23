//! `mk` is a tool to run commands as another user. It is similar to
//! [`doas`] or [`sudo`].
//!
//! [`doas`]: https://github.com/Duncaen/OpenDoas
//! [`sudo`]: https://www.sudo.ws/

#![deny(unsafe_code)]

#[macro_use]
pub mod utils;

pub mod auth;
pub mod cli;
pub mod config;
pub mod errors;
pub mod options;
pub mod permits;
pub mod policy;
pub mod prelude;
pub mod session;

pub use errors::*;
