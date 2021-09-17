//! Authenticated session tools.

use std::io;
use std::time::SystemTime;

use mk_pwd::Passwd;

use crate::auth::UserAuthenticator;
use crate::prelude::*;

mod rules;
mod state;

pub use rules::*;
pub use state::*;

/// Represents a recoverable authenticated user session.
///
/// A user session wraps around an authentication service to validate a user's account and run functions
/// in an authenticated session.
pub struct UserSession {
    /// Current state of the session.
    state: State,
    /// Authentication service to use.
    auth: Box<dyn UserAuthenticator>,
    /// Pre-defined session rules.
    rules: Rules,
}

impl UserSession {
    /// Create a new session for this user.
    pub fn new(auth: Box<dyn UserAuthenticator>, rules: Rules) -> Self {
        Self::with_state(auth, rules, State::new())
    }

    /// Create a new session from existing state.
    pub fn with_state(auth: Box<dyn UserAuthenticator>, rules: Rules, state: State) -> Self {
        Self { state, auth, rules }
    }

    /// Get the current state of this session.
    pub fn get_state(&self) -> &State {
        &self.state
    }

    /// Get the user this session is associated with.
    pub fn get_user(&self) -> &Passwd {
        self.auth.get_user()
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
        if !(self.auth.get_user() == target
            || self.rules.permitted.contains(&target.name)
            || self.rules.all_targets)
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("not permitted to run as user {}", target.name),
            )
            .into());
        }

        // Check if the user needs to be re-validated
        if !self.rules.no_auth {
            let mut need_auth = true;

            // Check if the session has exceeded its timeout
            if let Some(s) = self.state.last_used {
                if let Ok(dur) = SystemTime::now().duration_since(s) {
                    if let Some(t) = self.rules.get_timeout() {
                        need_auth = dur > t;
                    }
                }
            };

            if need_auth {
                self.auth.validate()?;
            }

            self.state.last_used = Some(SystemTime::now());
        }

        self.auth.session(session)
    }
}
