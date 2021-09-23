//! This holds everything together.

use std::cell::Cell;
use std::fs;
use std::io;
use std::os::unix::process::{parent_id, CommandExt};
use std::path::PathBuf;
use std::process::Command;

use nix::unistd::{getuid, User};

use crate::auth;
use crate::config::Config;
use crate::options::*;
use crate::permits::Permits;
use crate::policy::Policy;
use crate::prelude::*;
use crate::session::{State, UserSession};

pub struct App {
    session: UserSession,
    permits: Permits,
}

impl App {
    pub fn new(cfg: &Config) -> Result<Self> {
        let uid = getuid();
        let user = match User::from_uid(uid)? {
            Some(u) => u,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "could not find this user, for some reason",
                )
                .into())
            }
        };

        // Ignore configs if the user is root
        if uid.is_root() {
            let policy = Policy::root();
            let session = UserSession::new(
                auth::new(user, cfg.service, policy.auth.clone())?,
                policy.session.clone(),
            );

            return Ok(Self {
                session,
                permits: policy.permits.clone(),
            });
        }

        // Find a suitable policy and create a session
        if let Some(policy) = cfg.get_user_policy(&user)? {
            let session_state = Self::recover_session_state_or_new(&user)?;

            let session = UserSession::with_state(
                auth::new(user, cfg.service, policy.auth.clone())?,
                policy.session.clone(),
                session_state,
            );

            return Ok(Self {
                session,
                permits: policy.permits.clone(),
            });
        }

        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "no defined policy for this user",
        )
        .into())
    }

    /// Check if a user is allowed to run as a target.
    pub fn check(&self, target: &User) -> Result<()> {
        // ᕙ(⇀‸↼‵‵)ᕗ
        if !(self.session.get_user() == target
            || self.permits.targets.contains(&target.name)
            || self.permits.all_targets)
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("not permitted to run as user {}", target.name),
            )
            .into());
        }

        Ok(())
    }

    /// Run the appropriate method for given options.
    ///
    /// # Returns
    ///
    /// Exit status of the process run (if any).
    pub fn run(&mut self, options: MkOptions) -> Result<Option<i32>> {
        let res = match options {
            MkOptions::Command(cmd) => self.exec(cmd),
            MkOptions::Text(s) => {
                println!("{}", s);
                Ok(None)
            }
            _ => Ok(None),
        };

        // we'll probably log this later
        let _ = Self::save_session_state(&self.session);

        res
    }

    /// Execute a command with the given `options`.
    ///
    /// # Returns
    ///
    /// Exit status of the process run (if any).
    pub fn exec(&mut self, options: CommandOptions) -> Result<Option<i32>> {
        let exit = Cell::new(None);
        let target = &options.target;

        self.check(target)?;
        self.session.run(
            target,
            Box::new(|| -> Result<()> {
                let mut command = Command::new(&options.command[..]);

                command.uid(options.target.uid.as_raw());
                command.gid(options.target.gid.as_raw());

                command.args(options.args);

                // TODO: env preservation

                if let Some(c) = command.spawn()?.wait()?.code() {
                    let _ = &exit.set(Some(c));
                }

                Ok(())
            }),
        )??;

        Ok(exit.into_inner())
    }

    // Session related stuff

    /// Directory to which session state files are stored.
    const SESSION_DIR: &'static str = "/var/run/mk/sess";

    /// Try to save a session state to a file for later recovery.
    fn save_session_state(session: &UserSession) -> Result<()> {
        let mut path = PathBuf::new();
        path.push(Self::SESSION_DIR);

        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        utils::set_mode(&path, 0o600)?;

        path.push(format!("{}-{}", session.get_user().name, parent_id()));

        let mut f = fs::File::create(&path)?;
        session.get_state().try_dump(&mut f)?;

        utils::set_mode(&path, 0o600)?;
        Ok(())
    }

    /// Try to recover a session from its stored state a file. If a session could not be found,
    /// create a new one.
    fn recover_session_state_or_new(user: &User) -> Result<State> {
        let mut path = PathBuf::new();

        path.push(Self::SESSION_DIR);
        path.push(format!("{}-{}", user.name, parent_id()));

        if !path.exists() {
            return Ok(State::new());
        }

        let mut f = fs::File::open(path)?;
        State::try_recover(&mut f)
    }
}
