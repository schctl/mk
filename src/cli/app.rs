//! The `mk` app.

use std::cell::Cell;
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
            authenticator: auth::new(auth::Supported::Pwd)?,
        })
    }

    /// Run the appropriate method for given options.
    ///
    /// # Returns
    ///
    /// Exit status of the process run (if any).
    pub fn run(&mut self, options: MkOptions) -> MkResult<Option<i32>> {
        // clippy complains here despite `MkOptions` being `non_exhaustive`.

        #[allow(unreachable_patterns)]
        match options {
            MkOptions::Command(cmd) => return self.exec(cmd),
            MkOptions::Text(help) => {
                println!("{}", help);
            }
            MkOptions::Error(help) => {
                eprintln!("{}", help);
            }
            _ => {}
        }

        Ok(None)
    }

    /// Execute a command with the given `options`.
    pub fn exec(&mut self, options: CommandOptions) -> MkResult<Option<i32>> {
        // TODO:
        // check if `origin` is allowed to execute as `target` from the config.

        let exit = Cell::new(None);
        let origin = options.origin()?;

        let session = Box::new(|| -> MkResult<()> {
            let mut command = Command::new(options.command);

            // Set arguments
            command.args(options.args);

            // Clear environment and set new variables
            command.env_clear();

            if let Some(vars) = options.env {
                command.envs(vars);
            }

            // Set ids
            command.uid(options.target.uid);
            command.gid(options.target.gid);

            // Wait on child
            if let Some(c) = command.spawn()?.wait()?.code() {
                let _ = &exit.set(Some(c));
            }

            Ok(())
        });

        self.authenticator.session(&origin, session)?;
        Ok(exit.into_inner())
    }
}
