//! `mk` runtime options.

use std::collections::HashMap;

use crate::prelude::*;

/// Run a command as another user.
#[derive(Debug, Clone)]
pub struct CommandOptions {
    /// User to run the command as.
    pub target: mk_pwd::Passwd,
    /// Command to run.
    pub command: String,
    /// Arguments to pass to the command.
    pub args: Vec<String>,
    /// Environment variable mappings.
    pub env: Option<HashMap<String, String>>,
}

impl CommandOptions {
    /// Get the user that created the request. This is always the user who called this process.
    pub fn origin(&self) -> MkResult<mk_pwd::Passwd> {
        Ok(mk_pwd::Passwd::from_uid(util::get_uid())?)
    }
}

/// All runtime options for `mk`.
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum MkOptions {
    None,
    Command(CommandOptions),
    Text(String),
    Error(String),
}
