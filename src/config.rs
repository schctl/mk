//! `mk` configurations.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use crate::auth::AuthService;
use crate::policy::Policy;
use crate::prelude::*;

/// Global `mk` configurations.
#[readonly::make]
#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Config {
    /// All defined policies.
    #[serde(default = "HashMap::new")]
    pub policies: HashMap<String, Policy>,
    /// User policies. Values correspond to a predefined policy.
    #[serde(default = "HashMap::new")]
    pub users: HashMap<String, String>,
    /// Default authentication service to use.
    #[serde(default = "AuthService::default")]
    pub service: AuthService,
}

impl Config {
    /// Try to read configurations from a file.
    #[inline]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        toml::from_str(&fs::read_to_string(path)?[..])
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e).into())
    }

    #[must_use]
    #[inline]
    pub fn get_user_policy(&self, user: &str) -> Option<&Policy> {
        self.users.get(user).and_then(|s| self.policies.get(s))
    }
}
