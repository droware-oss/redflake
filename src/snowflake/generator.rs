use crate::snowflake::id::{EPOCH, MAX_SEQ};
use crate::snowflake::SnowflakeId;
use std::fmt::Debug;
use std::hint::spin_loop;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::SystemTime;

pub struct SnowflakeGenerator {
    machine_id: u8,
    last_id: AtomicI64,
}

impl SnowflakeGenerator {
    pub fn new(machine_id: u8) -> Self {
        Self {
            machine_id,
            last_id: AtomicI64::new(0),
        }
    }

    pub fn next_id(&self) -> Result<i64, String> {
        loop {
            let last_id = self.last_id.load(Ordering::Acquire);
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
                machine: self.machine_id,
                sequence,
            });
            if let Ok(_) = self.last_id.compare_exchange_weak(
                last_id,
                next_id,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                return Ok(next_id);
            } else {
                spin_loop();
            }
        }
    }
}

impl Debug for SnowflakeGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnowflakeGenerator")
            .field("machine_id", &self.machine_id)
            .finish()
    }
}
