//! Authenticated session tools.

use std::time::SystemTime;

use nix::unistd::User;

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
    #[must_use]
    pub fn new(auth: Box<dyn UserAuthenticator>, rules: Rules) -> Self {
        Self::with_state(auth, rules, State::new())
    }

    /// Create a new session from existing state.
    #[must_use]
    pub fn with_state(auth: Box<dyn UserAuthenticator>, rules: Rules, state: State) -> Self {
        Self { state, auth, rules }
    }

    /// Get the current state of this session.
    #[must_use]
    #[inline]
    pub fn get_state(&self) -> &State {
        &self.state
    }

    /// Get the user this session is associated with.
    #[must_use]
    #[inline]
    pub fn get_user(&self) -> &User {
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
        target: &User,
        session: Box<dyn FnOnce() -> Result<()> + 'a>,
    ) -> Result<Result<()>> {
        // Check if the user needs to be re-validated
        if !self.rules.no_auth {
            let mut need_auth = true;

            // Check if the session has exceeded its timeout
            if let Some(s) = self.state.last_used {
                if let Ok(dur) = SystemTime::now().duration_since(s) {
                    if let Some(t) = self.rules.refresh {
                        need_auth = dur > t;
                    }
                }
            };

            if need_auth {
                self.auth.validate()?;
            }

            self.state.use_now();
        }

        self.auth.session(session, target)
    }
}
