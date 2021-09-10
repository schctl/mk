//! `mk` runtime options.

use std::collections::HashMap;

/// Run a command as another user.
#[derive(Debug, Clone)]
pub struct CommandOptions {
    /// Requested user to run the command as.
    pub target: mk_pwd::Passwd,
    /// Command to run.
    pub command: String,
    /// Arguments to pass to the command.
    pub args: Vec<String>,
    /// Environment variable mappings.
    pub env: Option<HashMap<String, String>>,
}

/// All runtime options for `mk`.
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum MkOptions {
    None,
    Command(CommandOptions),
    Text(String),
}
