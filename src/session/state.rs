use std::io::prelude::*;
use std::time::SystemTime;

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use mk_common::*;

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

    /// Update the session's last time of use.
    #[inline]
    pub fn use_now(&mut self) {
        self.last_used = Some(SystemTime::now());
    }

    /// Try to recover a session's state from a reader.
    pub fn try_recover<T: Read>(reader: &mut T) -> Result<Self> {
        let cookie = reader.read_i64::<NativeEndian>()?;

        let last_used =
            de_duration(cookie, DurationResolution::Minutes).map(|d| SystemTime::UNIX_EPOCH + d);

        Ok(Self { last_used })
    }

    /// Try to write a session's state into a writer.
    ///
    /// # Serialization format
    ///
    /// Fields are serialized **in order**. There could be more fields added in the future.
    ///
    /// | Field       | Type |
    /// |-------------|------|
    /// | `last_used` | i64  |
    pub fn try_dump<T: Write>(&self, writer: &mut T) -> Result<usize> {
        let cookie = ser_duration(
            &self
                .last_used
                .and_then(|d| d.duration_since(SystemTime::UNIX_EPOCH).ok()),
            DurationResolution::Minutes,
        );

        writer.write_i64::<NativeEndian>(cookie)?;

        Ok(8)
    }
}
