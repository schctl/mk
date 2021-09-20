//! `mk` configurations.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;

use crate::auth::AuthService;
use crate::policy::Policy;
use crate::prelude::*;

/// Global `mk` configurations.
#[readonly::make]
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Config {
    /// All defined policies.
    #[serde(default = "HashMap::new")]
    pub policies: HashMap<String, Policy>,
    /// User policies. Values correspond to a predefined policy.
    pub users: HashMap<String, String>,
    /// Default authentication service to use.
    #[serde(default = "AuthService::default")]
    pub service: AuthService,
}

impl Config {
    /// Try to read configurations from a file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut f = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("could not find configuration file: {}", e),
                )
                .into())
            }
        };

        let mut contents = String::new();
        f.read_to_string(&mut contents)?;

        match toml::from_str(&contents[..]) {
            Ok(c) => Ok(c),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e).into()),
        }
    }

    pub fn get_user_policy(&self, user: &str) -> Option<&Policy> {
        if let Some(user_policy) = self.users.get(user) {
            if let Some(policy) = self.policies.get(user_policy) {
                return Some(policy);
            }
        }

        None
    }
}
