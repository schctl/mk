//! This holds everything together.

use std::cell::Cell;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;

use mk_common::get_uid;

use crate::auth;
use crate::config::Config;
use crate::options::*;
use crate::prelude::*;
use crate::session::{config::SessionConfig, UserSession};

pub struct App {
    session: UserSession,
}

impl App {
    pub fn new(cfg: Config) -> Result<Self> {
        let user = mk_pwd::Passwd::from_uid(get_uid())?;

        if let Some(c) = cfg.session.get(&user.name) {
            Ok(Self {
                session: UserSession::recover_or_new(SessionConfig {
                    auth: auth::new(user, cfg.authenticator)?,
                    rules: c.clone(),
                })?,
            })
        } else {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, "no found session").into())
        }
    }

    /// Run the appropriate method for given options.
    ///
    /// # Returns
    ///
    /// Exit status of the process run (if any).
    pub fn run(&mut self, options: MkOptions) -> Result<Option<i32>> {
        match options {
            MkOptions::Command(cmd) => return self.exec(cmd),
            MkOptions::Text(help) => println!("{}", help),
            _ => {}
        }

        Ok(None)
    }

    /// Execute a command with the given `options`.
    ///
    /// # Returns
    ///
    /// Exit status of the process run (if any).
    pub fn exec(&mut self, options: CommandOptions) -> Result<Option<i32>> {
        let target = &options.target;

        let exit = Cell::new(None);

        let session: Box<dyn FnOnce() -> Result<()>> = Box::new(|| -> Result<()> {
            let mut command = Command::new(&options.command[..]);

            command.uid(options.target.uid);
            command.gid(options.target.gid);

            command.args(options.args);

            // TODO: env preservation

            // Wait on child
            if let Some(c) = command.spawn()?.wait()?.code() {
                let _ = &exit.set(Some(c));
            }

            Ok(())
        });

        self.session.run(target, session)??;

        // we'll probably log this later
        let _ = self.session.store_to_file();

        Ok(exit.into_inner())
    }
}
