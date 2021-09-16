//! Authenticated session tools.

use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

use mk_common::*;

use crate::prelude::*;
use crate::util::set_mode;

pub mod config;

/// Represents a recoverable authenticated user session.
///
/// A user session wraps around an authentication service to validate a user's account and run functions
/// in an authenticated session.
pub struct UserSession {
    /// Current state of the session.
    state: config::State,
    /// Pre-defined session rules.
    cfg: config::SessionConfig,
}

impl UserSession {
    /// Session storage directory.
    pub const STORAGE_DIR: &'static str = "/var/run/mk/sess";

    /// Private constructor.
    fn __new(state: config::State, cfg: config::SessionConfig) -> Result<Self> {
        Ok(Self { state, cfg })
    }

    /// Create a new session for this user.
    pub fn new(cfg: config::SessionConfig) -> Result<Self> {
        Self::__new(config::State::new(), cfg)
    }

    /// Try to recover a session from its stored state a file. If a session could not be found,
    /// create a new one.
    pub fn recover_or_new(cfg: config::SessionConfig) -> Result<Self> {
        let mut path = PathBuf::new();

        path.push(Self::STORAGE_DIR);
        path.push(format!("{}-{}", cfg.auth.get_user().name, get_parent_pid()));

        if !path.exists() {
            return Self::new(cfg);
        }

        let mut f = fs::File::open(path)?;
        let state = config::State::try_recover(&mut f)?;
        Self::__new(state, cfg)
    }

    /// Try to store this session's state into a file.
    pub fn store_to_file(&self) -> Result<()> {
        let mut path = PathBuf::new();
        path.push(Self::STORAGE_DIR);

        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        util::set_mode(&path, 0o600)?;

        path.push(format!(
            "{}-{}",
            self.cfg.auth.get_user().name,
            get_parent_pid()
        ));

        let mut f = fs::File::create(&path)?;
        self.state.try_dump(&mut f)?;

        set_mode(&path, 0o600)?;
        Ok(())
    }

    /// Validate a user's account and run a function in an authenticated session.
    ///
    /// # Returns
    ///
    /// If successful, the function returns an [`Ok`] containing the result of the function.
    ///
    /// # Errors
    ///
    /// This function fails if the underlying service was unable to start or close a session, or if
    /// the session rules do not permit this action.
    pub fn run<'a>(
        &mut self,
        target: &mk_pwd::Passwd,
        session: Box<dyn FnOnce() -> Result<()> + 'a>,
    ) -> Result<Result<()>> {
        // ᕙ(⇀‸↼‵‵)ᕗ
        if !(self.cfg.auth.get_user() == target
            || self.cfg.rules.get_permitted().contains(&target.name))
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("not permitted to run as user {}", target.name),
            )
            .into());
        }

        // Check if the user needs to be re-validated
        if !self.cfg.rules.get_no_auth() {
            // Don't worry about the session lifetime if no_auth is allowed.
            if let Some(s) = self.state.last_active {
                if let Ok(dur) = SystemTime::now().duration_since(s) {
                    if dur > self.cfg.rules.get_timeout() {
                        self.cfg.auth.validate()?;
                    }
                }
            } else {
                self.cfg.auth.validate()?;
            }

            self.state.last_active = Some(SystemTime::now());
        }

        self.cfg.auth.session(session)
    }
}
