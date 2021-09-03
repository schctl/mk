//! `mk` runtime options.

use std::collections::HashMap;

use crate::prelude::*;

/// Command execution options.
#[derive(Debug, Clone)]
pub struct CommandOptions {
    pub target: mk_pwd::Passwd,
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
}

impl CommandOptions {
    pub fn origin(&self) -> MkResult<mk_pwd::Passwd> {
        Ok(mk_pwd::Passwd::from_uid(util::get_uid())?)
    }
}

/// All runtime options for `mk`.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum MkOptions {
    Command(CommandOptions),
    Help(String),
}
