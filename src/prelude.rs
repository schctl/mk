//! Useful re-exports.

pub use crate::errors::MkError;
pub use crate::util;

pub type MkResult<T> = Result<T, MkError>;

pub const VERSION: &str = "0.0.1";
pub const SERVICE_NAME: &str = "mk";
pub const DESCRIPTION: &str = "Run commands as another user";
