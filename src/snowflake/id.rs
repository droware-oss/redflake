use chrono::{DateTime, SecondsFormat};
use std::fmt::Display;

/// Custom epoch: 2000-01-01 00:00:00 UTC
pub const EPOCH: u64 = 946684800000;

/// Maximum sequence value (12 bits)
pub const MAX_SEQ: u16 = 0xfff;

pub struct SnowflakeId {
    pub timestamp: u64,
    pub machine: u8,
    pub sequence: u16,
}

impl Display for SnowflakeId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Snowflake(Time: {}, Machine: {}, Sequence: {}) -> ID({})",
            DateTime::from_timestamp_millis((self.timestamp + EPOCH) as i64)
                .unwrap()
                .to_rfc3339_opts(SecondsFormat::Millis, false),
            self.machine,
            self.sequence,
            i64::from(self)
        )
    }
}

impl From<i64> for SnowflakeId {
    fn from(id: i64) -> Self {
        SnowflakeId {
            timestamp: (id >> 20) as u64,
            machine: (id >> 12) as u8,
            sequence: id as u16 & MAX_SEQ,
        }
    }
}

impl From<&SnowflakeId> for i64 {
    fn from(id: &SnowflakeId) -> Self {
        (id.timestamp << 20 | (id.machine as u64) << 12 | id.sequence as u64) as i64
    }
}

impl From<SnowflakeId> for i64 {
    fn from(id: SnowflakeId) -> Self {
        i64::from(&id)
    }
}
