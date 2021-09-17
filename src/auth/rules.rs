//! Authenticator configurations.

use std::time::Duration;

use crate::prelude::*;

/// Default field values.
pub(crate) mod defaults {
    use super::AuthService;
    use super::Rules;

    #[inline]
    pub const fn ty() -> AuthService {
        #[cfg(feature = "pam")]
        return AuthService::Pam;

        #[cfg(not(feature = "pam"))]
        return AuthService::Pwd;
    }

    #[inline]
    pub const fn timeout() -> i64 {
        2
    }

    #[inline]
    pub const fn rules() -> Rules {
        Rules { timeout: timeout() }
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

/// Predefined rules for a user session.
#[readonly::make]
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Rules {
    /// Validation timeout.
    #[serde(default = "defaults::timeout")]
    timeout: i64,
}

impl Rules {
    /// Interpret the serialized timeout as a [`Duration`].
    #[inline]
    pub fn get_timeout(&self) -> Option<Duration> {
        utils::duration_from_minutes(self.timeout)
    }
}
