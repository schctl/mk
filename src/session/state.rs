use std::io::prelude::*;
use std::time::{Duration, SystemTime};

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

use crate::prelude::*;

/// Internal, recoverable session state.
#[derive(Debug)]
pub struct State {
    /// The last time at which this session was active.
    pub last_used: Option<SystemTime>,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    #[must_use]
    pub fn new() -> Self {
        Self { last_used: None }
    }

    /// Try to recover a session's state from a reader.
    pub fn try_recover<T: Read>(reader: &mut T) -> Result<Self> {
        let cookie = reader.read_i64::<NativeEndian>()?;

        let last_used = if cookie < 0 {
            None
        } else {
            Some(SystemTime::UNIX_EPOCH + Duration::from_secs(cookie as u64))
        };

        Ok(Self { last_used })
    }

    /// Try to write a session's state into a writer.
    ///
    /// # Serialization format
    ///
    /// Fields serialized **in order**. There will probably be more fields added in the future.
    ///
    /// | Field       | Type |
    /// |-------------|------|
    /// | last_used | i64  |
    pub fn try_dump<T: Write>(&self, writer: &mut T) -> Result<usize> {
        let cookie = match self.last_used {
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
