//! User session configuration and state.

use std::time::Duration;

use crate::prelude::*;

/// Default field values.
pub(crate) mod defaults {
    use super::*;

    #[inline]
    pub const fn refresh() -> Option<Duration> {
        None
    }

    #[inline]
    pub const fn no_auth() -> bool {
        false
    }
}

/// Predefined rules for a user session.
#[readonly::make]
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Rules {
    /// Maximum inactive duration after which the session must re-validate its user.
    #[serde(with = "utils::timeout_serializer")]
    #[serde(default = "defaults::refresh")]
    pub refresh: Option<Duration>,
    /// Allow session to forego user validation.
    #[serde(default = "defaults::no_auth")]
    pub no_auth: bool,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            refresh: defaults::refresh(),
            no_auth: defaults::no_auth(),
        }
    }
}

impl Rules {
    /// Root user session overrides.
    #[must_use]
    pub fn root() -> Self {
        Self {
            no_auth: true,
            ..Self::default()
        }
    }
}
