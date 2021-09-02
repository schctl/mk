//! Environment setup utilities for running a command.

use std::collections::HashMap;
use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::auth::{self, Authenticator};
use crate::config::Config;
use crate::options::MkOptions;
use crate::prelude::*;

/// The execution environment.
pub struct Env {
    authenticator: Box<dyn Authenticator>,
    origin: mk_pwd::Passwd,
    _config: Config,
}

impl Env {
    /// Create a new execution environment with the current user.
    #[must_use]
    pub fn new(_config: Config) -> Self {
        Self {
            // This will be created after reading the config later.
            authenticator: Box::new(auth::pam::PamAuthenticator::new()),
            origin: mk_pwd::Passwd::from_uid(util::get_uid()).unwrap(),
            _config,
        }
    }

    /// Verify that the current user is authenticated and is allowed to run as the target.
    /// `Err` is returned if the verification failed.
    pub fn verify(&mut self, _target: &mk_pwd::Passwd) -> MkResult<()> {
        self.authenticator.authenticate(&self.origin)?;

        // TODO:
        // check if `origin` is allowed to execute as `target` from the config.

        return Ok(());
    }

    /// Run the appropriate method for given options.
    pub fn run(&mut self, options: MkOptions) -> MkResult<()> {
        match options {
            MkOptions::Command(cmd) => Err(self.exec(
                &cmd.command[..],
                cmd.args,
                cmd.env,
                &mk_pwd::Passwd::from_name(&cmd.target[..])?,
            )),
            MkOptions::Help(help) => {
                println!("{}", help);
                Ok(())
            }
        }
    }

    /// Run a command as a `target` user, if the environment is verified.
    pub fn exec(
        &mut self,
        cmd: &str,
        args: Vec<String>,
        env: Option<HashMap<String, String>>,
        target: &mk_pwd::Passwd,
    ) -> MkError {
        match self.verify(&target) {
            Ok(_) => {
                let mut command = Command::new(cmd);

                // Set arguments
                command.args(args);

                // Clear environment and set new variables
                if let Some(vars) = env {
                    command.env_clear();
                    command.envs(vars);
                }

                // Set ids
                command.uid(target.uid);
                command.gid(target.gid);

                // Execute the command
                command.exec().into()
            }
            Err(e) => e,
        }
    }
}
