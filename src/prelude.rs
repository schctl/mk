//! Useful re-exports.

pub use crate::errors::MkError;
pub use crate::util;

pub type MkResult<T> = Result<T, MkError>;

pub const SERVICE_NAME: &str = "mk";
