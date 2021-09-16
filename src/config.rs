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
    pub session: HashMap<String, session::config::Rules>,
    /// Global authenticator configuration.
    pub authenticator: auth::config::AuthConfig,
}

impl Config {
    /// Try to read configurations from a file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut f = fs::File::open(path)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;

        match toml::from_str(&contents[..]) {
            Ok(c) => Ok(c),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e).into()),
        }
    }
}
