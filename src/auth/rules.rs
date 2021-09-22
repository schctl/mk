//! Authenticator configurations.

use std::time::Duration;

use crate::prelude::*;

/// Default field values.
pub(crate) mod defaults {
    use super::*;

    #[inline]
    pub const fn timeout() -> Option<Duration> {
        Some(Duration::from_secs(120))
    }
}

/// All supported authentication services.
#[allow(unused)]
#[non_exhaustive]
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Copy)]
pub enum AuthService {
    /// Authentication using PAM.
    #[cfg(feature = "pam")]
    Pam,
    /// Authentication using the system password database.
    Pwd,
}

impl Default for AuthService {
    fn default() -> Self {
        #[cfg(feature = "pam")]
        return AuthService::Pam;

        #[cfg(not(feature = "pam"))]
        return AuthService::Pwd;
    }
}

/// Predefined rules for a user session.
#[readonly::make]
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Rules {
    /// Validation timeout.
    #[serde(with = "utils::timeout_serializer")]
    #[serde(default = "defaults::timeout")]
    timeout: Option<Duration>,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            timeout: defaults::timeout(),
        }
    }
}
