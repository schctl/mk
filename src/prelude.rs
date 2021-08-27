//! Useful re-exports.

pub use crate::errors::MkError;

pub type MkResult<T> = Result<T, MkError>;
