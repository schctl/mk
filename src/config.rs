//! `mk` configurations.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;

use crate::auth;
use crate::prelude::*;
use crate::session;

/// Global `mk` configurations.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Config {
    /// Per user session rules.
    #[serde(default = "HashMap::new")]
    pub session: HashMap<String, session::Rules>,
    /// Global authenticator configuration.
    #[serde(rename = "auth-rules")]
    #[serde(default = "auth::defaults::rules")]
    pub auth: auth::Rules,
    /// Default authentication service to use.
    #[serde(default = "auth::defaults::ty")]
    pub service: auth::AuthService,
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
}
