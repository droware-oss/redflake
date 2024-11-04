use chrono::{DateTime, SecondsFormat};
use std::fmt::Display;
use std::hint::spin_loop;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::SystemTime;
use tokio::sync::SetOnce;

// 2000-01-01 00:00:00+00:00
const EPOCH: u64 = 946684800000;

const MAX_SEQ: u16 = 0xfff;

pub static MACHINE: SetOnce<u8> = SetOnce::const_new();

static LAST_ID: AtomicI64 = AtomicI64::new(0);

pub fn next_id() -> Result<i64, String> {
    loop {
        let last_id = LAST_ID.load(Ordering::Acquire);
        let id = SnowflakeId::from(last_id);
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
            - EPOCH;
        let mut sequence = 0;
        if now < id.timestamp {
            return Err("Clock moved backwards".to_string());
        } else if now == id.timestamp {
            sequence = id.sequence + 1;
            if sequence > MAX_SEQ {
                spin_loop();
                continue;
            }
        }
        let next_id = i64::from(&SnowflakeId {
            timestamp: now,
            machine: *MACHINE.get().unwrap(),
            sequence,
        });
        if let Ok(_) =
            LAST_ID.compare_exchange_weak(last_id, next_id, Ordering::Release, Ordering::Relaxed)
        {
            return Ok(next_id);
        } else {
            spin_loop();
        }
    }
}

pub struct SnowflakeId {
    timestamp: u64,
    machine: u8,
    sequence: u16,
}

impl SnowflakeId {}

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
