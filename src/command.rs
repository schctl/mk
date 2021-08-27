//! Command execution tools.

#![allow(unused)]

use std::os::unix::process::CommandExt;
use std::process::{Command, Output};

use crate::auth::Authenticator;
use crate::env::Env;

use crate::prelude::*;

/// Command execution wrapper.
pub struct CommandExecutor {
    env: Env,
    authenticator: Box<dyn Authenticator>,
}

impl CommandExecutor {
    /// Construct a new command executor for this environment.
    #[must_use]
    pub fn new(env: Env, authenticator: Box<dyn Authenticator>) -> Self {
        Self { env, authenticator }
    }

    /// Verify if the environment is valid for command execution.
    fn verify_env(&mut self) -> Result<(), MkError> {
        // First, check if the user is authenticated.
        self.authenticator.authenticate(&self.env.origin)?;

        // Check if `user` is permitted to run as `target`.
        // TODO

        Ok(())
    }

    /// Get the execution environment.
    pub fn get_env(&self) -> &Env {
        &self.env
    }

    /// Get a mutable reference to the execution environment.
    pub fn get_env_mut(&mut self) -> &mut Env {
        &mut self.env
    }

    /// Build a command to execute.
    pub fn build_cmd(&self, cmd: &str) -> Command {
        let mut command = Command::new(cmd);

        // Pass arguments
        command.args(self.env.get_args());

        // Clear environment and set new variables
        command.env_clear();
        command.envs(self.env.get_vars());

        // Set ids
        command.uid(self.env.target.uid);
        command.uid(self.env.target.gid);

        command
    }

    /// Execute a command and replace the current process image with the new one.
    pub fn exec_cmd(mut self, cmd: &str) -> MkResult<!> {
        self.verify_env()?;
        Err(self.build_cmd(cmd).exec().into())
    }

    /// Execute a command as a child process, wait for it to finish, and collect its output.
    pub fn exec_process(&mut self, cmd: &str) -> MkResult<Output> {
        self.verify_env()?;
        Ok(self.build_cmd(cmd).output()?)
    }
}
