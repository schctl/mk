//! `mk` runtime options.

use std::path::PathBuf;

use mk_pwd::Passwd;

/// Run a command as another user.
#[derive(Debug, Clone)]
pub struct CommandOptions {
    /// Requested user to run the command as.
    pub target: Passwd,
    /// Command to run.
    pub command: String,
    /// Arguments to pass to the command.
    pub args: Vec<String>,
    /// Environment variable mappings.
    pub preserve_env: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct EditOptions {
    /// Requested user to edit the file as.
    pub target: Passwd,
    /// Path of file to edit.
    pub path: PathBuf,
}

/// All runtime options for `mk`.
#[non_exhaustive]
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum MkOptions {
    None,
    Command(CommandOptions),
    Edit(EditOptions),
    Text(String),
}
