//! A user policy.

use crate::auth;
use crate::session;

/// A policy is a definition for session and authenticator configurations.
#[readonly::make]
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Policy {
    /// Session configuration.
    #[serde(default = "session::Rules::default")]
    pub session: session::Rules,
    /// Authenticator configuration.
    #[serde(default = "auth::Rules::default")]
    pub auth: auth::Rules,
}
