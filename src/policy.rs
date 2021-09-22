//! A user policy.

use crate::auth;
use crate::permits;
use crate::session;

/// A policy is a common definition for all actions and configurations for a user or group.
#[readonly::make]
#[derive(Debug, Default, serde::Deserialize, serde::Serialize, Clone)]
pub struct Policy {
    /// Actions this policy allows for
    #[serde(default = "permits::Permits::default")]
    pub permits: permits::Permits,
    /// Session configuration.
    #[serde(default = "session::Rules::default")]
    pub session: session::Rules,
    /// Authenticator configuration.
    #[serde(default = "auth::Rules::default")]
    pub auth: auth::Rules,
}

impl Policy {
    /// Policy for the root user.
    #[must_use]
    pub fn root() -> Self {
        Self {
            permits: permits::Permits::root(),
            session: session::Rules::root(),
            ..Self::default()
        }
    }
}
