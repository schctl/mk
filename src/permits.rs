//! User and group permits.

mod defaults {
    #[inline]
    pub const fn targets() -> Vec<String> {
        Vec::new()
    }

    #[inline]
    pub const fn all_targets() -> bool {
        false
    }
}

/// Definitions for all actions a user or group is allowed to do.
#[readonly::make]
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Permits {
    /// Permitted targets.
    #[serde(default = "defaults::targets")]
    pub targets: Vec<String>,
    /// Permit running as all targets. This will cause the session to ignore the `permitted` field.
    #[serde(rename = "all-targets")]
    #[serde(default = "defaults::all_targets")]
    pub all_targets: bool,
}

impl Default for Permits {
    fn default() -> Self {
        Self {
            targets: defaults::targets(),
            all_targets: defaults::all_targets(),
        }
    }
}

impl Permits {
    /// Permit overrides for the root user.
    #[must_use]
    pub fn root() -> Self {
        Self {
            all_targets: true,
            ..Self::default()
        }
    }
}
