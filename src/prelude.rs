//! Useful re-exports.

pub use crate::errors::Error;
pub use crate::utils;

pub type Result<T> = core::result::Result<T, Error>;

pub const VERSION: &str = "0.0.1";
pub const SERVICE_NAME: &str = "mk";
pub const DESCRIPTION: &str = "Run commands as another user";
