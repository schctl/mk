//! The `mk` app.

use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::auth::{self, Authenticator};
use crate::config::Config;
use crate::options::*;
use crate::prelude::*;

pub struct App {
    _config: Config,
    authenticator: Box<dyn Authenticator>,
}

impl App {
    pub fn new(_config: Config) -> MkResult<Self> {
        Ok(Self {
            _config,
            // TODO: this will be parsed from `_config` later on.
            #[cfg(feature = "pam")]
            authenticator: auth::new(auth::Supported::Pam)?,
            #[cfg(not(feature = "pam"))]
            authenticator: auth::new(auth::Supported::Passwd)?,
        })
    }

    /// Run the appropriate method for given options.
    pub fn run(&mut self, options: MkOptions) -> MkResult<()> {
        match options {
            MkOptions::Command(cmd) => {
                if let Err(e) = self.exec(cmd) {
                    Err(e)
                } else {
                    Ok(())
                }
            }
            MkOptions::Help(help) => {
                println!("{}", help);
                Ok(())
            }
        }
    }

    /// Execute a command with the given `options`.
    pub fn exec(&mut self, options: CommandOptions) -> MkResult<!> {
        self.authenticator.authenticate(&options.origin()?)?;

        // TODO:
        // check if `origin` is allowed to execute as `target` from the config.

        let mut command = Command::new(options.command);

        // Set arguments
        command.args(options.args);

        // Clear environment and set new variables
        if let Some(vars) = options.env {
            command.env_clear();
            command.envs(vars);
        }

        // Set ids
        command.uid(options.target.uid);
        command.gid(options.target.gid);

        // Execute the command
        Err(command.exec().into())
    }
}
