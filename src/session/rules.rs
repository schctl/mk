//! User session configuration and state.

use std::time::Duration;

use crate::prelude::*;

/// Default field values.
pub(crate) mod defaults {
    #[inline]
    pub const fn timeout() -> i64 {
        -1
    }

    #[inline]
    pub const fn no_auth() -> bool {
        false
    }

    #[inline]
    pub const fn permitted() -> Vec<String> {
        Vec::new()
    }

    #[inline]
    pub const fn all_targets() -> bool {
        false
    }
}

/// Predefined rules for a user session.
#[readonly::make]
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Rules {
    /// Maximum inactive duration after which the session must re-validate its user.
    #[serde(default = "defaults::timeout")]
    timeout: i64,
    /// Allow session to forego user validation.
    #[serde(default = "defaults::no_auth")]
    pub no_auth: bool,
    /// Permitted targets.
    #[serde(default = "defaults::permitted")]
    pub permitted: Vec<String>,
    /// Permit running as all targets. This will cause the session to ignore the `permitted` field.
    #[serde(rename = "all-targets")]
    #[serde(default = "defaults::all_targets")]
    pub all_targets: bool,
}

impl Rules {
    /// Interpret the serialized timeout as a [`Duration`].
    #[inline]
    pub fn get_timeout(&self) -> Option<Duration> {
        utils::duration_from_minutes(self.timeout)
    }
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            timeout: defaults::timeout(),
            no_auth: defaults::no_auth(),
            permitted: defaults::permitted(),
            all_targets: defaults::all_targets(),
        }
    }
}
