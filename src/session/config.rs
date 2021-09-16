//! User session configuration and state.

use std::io::prelude::*;
use std::time::{Duration, SystemTime};

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

use crate::auth;
use crate::prelude::*;

/// Internal, recoverable session state.
#[derive(Debug)]
pub struct State {
    /// The last time at which this session was active.
    pub last_active: Option<SystemTime>,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    #[must_use]
    pub fn new() -> Self {
        Self { last_active: None }
    }

    // Serialization format
    // --------------------

    /// Try to recover a session's state from a reader.
    pub fn try_recover<T: Read>(reader: &mut T) -> Result<Self> {
        let cookie = reader.read_i64::<NativeEndian>()?;

        let last_active = if cookie < 0 {
            None
        } else {
            Some(SystemTime::UNIX_EPOCH + Duration::from_secs(cookie as u64))
        };

        Ok(Self { last_active })
    }

    /// Try to write a session's state into a writer.
    ///
    /// # Serialization format
    ///
    /// Fields serialized **in order**. There will probably be more fields added in the future.
    ///
    /// | Field       | Type |
    /// |-------------|------|
    /// | last_active | i64  |
    pub fn try_dump<T: Write>(&self, writer: &mut T) -> Result<usize> {
        let cookie = match self.last_active {
            Some(s) => match s.duration_since(SystemTime::UNIX_EPOCH) {
                Ok(d) => d.as_secs() as i64,
                Err(_) => -1,
            },
            None => -1,
        };

        writer.write_i64::<NativeEndian>(cookie)?;

        Ok(8)
    }
}

/// Predefined rules for a user session.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Rules {
    /// User session timeout.
    timeout: u64,
    /// Allow session to forego user validation.
    no_auth: bool,
    /// Permitted targets.
    permitted: Vec<String>,
}

impl Rules {
    /// Get maximum inactive duration after which the session must re-validate its user.
    pub fn get_timeout(&self) -> Duration {
        Duration::from_secs(self.timeout * 60)
    }

    /// Check if this session's rules allows it to forego validation.
    pub fn get_no_auth(&self) -> bool {
        self.no_auth
    }

    /// Get a list of permitted users this session is allowed to run as.
    pub fn get_permitted(&self) -> &Vec<String> {
        &self.permitted
    }
}

/// Required information used by a user session.
pub struct SessionConfig {
    /// Authentication service to use.
    pub auth: Box<dyn auth::UserAuthenticator>,
    /// Session rules.
    pub rules: Rules,
}
