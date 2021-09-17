//! This holds everything together.

use std::cell::Cell;
use std::fs;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use mk_common::{get_parent_pid, get_uid};
use mk_pwd::Passwd;

use crate::auth;
use crate::config::Config;
use crate::options::*;
use crate::prelude::*;
use crate::session::{State, UserSession};

pub struct App {
    session: UserSession,
}

impl App {
    /// Directory to which session state files are stored.
    const SESSION_DIR: &'static str = "/var/run/mk/sess";

    pub fn new(mut cfg: Config) -> Result<Self> {
        let user = Passwd::from_uid(get_uid())?;

        if let Some(c) = cfg.session.remove_entry(&user.name) {
            let session_state = Self::recover_session_state_or_new(&user)?;

            let session = UserSession::with_state(
                auth::new(user, cfg.service, cfg.auth)?,
                c.1,
                session_state,
            );

            Ok(Self { session })
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
            MkOptions::Text(s) => println!("{}", s),
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

            if let Some(c) = command.spawn()?.wait()?.code() {
                let _ = &exit.set(Some(c));
            }

            Ok(())
        });

        self.session.run(target, session)??;

        // we'll probably log this later
        let _ = Self::save_session_state(&self.session);

        Ok(exit.into_inner())
    }

    /// Try to save a session state to a file for later recovery.
    fn save_session_state(session: &UserSession) -> Result<()> {
        let mut path = PathBuf::new();
        path.push(Self::SESSION_DIR);

        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        utils::set_mode(&path, 0o600)?;

        path.push(format!("{}-{}", session.get_user().name, get_parent_pid()));

        let mut f = fs::File::create(&path)?;
        session.get_state().try_dump(&mut f)?;

        utils::set_mode(&path, 0o600)?;
        Ok(())
    }

    /// Try to recover a session from its stored state a file. If a session could not be found,
    /// create a new one.
    fn recover_session_state_or_new(user: &Passwd) -> Result<State> {
        let mut path = PathBuf::new();

        path.push(Self::SESSION_DIR);
        path.push(format!("{}-{}", user.name, get_parent_pid()));

        if !path.exists() {
            return Ok(State::new());
        }

        let mut f = fs::File::open(path)?;

        State::try_recover(&mut f)
    }
}
