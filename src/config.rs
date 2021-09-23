//! `mk` configurations.

use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::io;
use std::path::Path;

use nix::unistd;

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
    /// Group policies. Values correspond to a predefined policy.
    #[serde(default = "HashMap::new")]
    pub groups: HashMap<String, String>,
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
    pub fn get_user_policy(&self, user: &unistd::User) -> Result<Option<&Policy>> {
        // Get user specific policy
        if let Some(p) = self
            .users
            .get(&user.name)
            .and_then(|s| self.policies.get(s))
        {
            return Ok(Some(p));
        }

        // https://docs.rs/nix/0.22.1/nix/unistd/fn.getgrouplist.html
        #[cfg(not(target_os = "macos"))]
        // Get policy for any supplementary groups this user might be in
        for g in unistd::getgrouplist(CString::new(&user.name[..])?.as_c_str(), user.gid)? {
            if let Some(grp) = unistd::Group::from_gid(g)? {
                if let Some(p) = self
                    .groups
                    .get(&grp.name)
                    .and_then(|s| self.policies.get(s))
                {
                    return Ok(Some(p));
                }
            }
        }

        Ok(None)
    }
}
