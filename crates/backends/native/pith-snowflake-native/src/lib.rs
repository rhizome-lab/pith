//! Native snowflake ID implementation.

use pith_snowflake::{Snowflake, SnowflakeError, SnowflakeId};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Twitter snowflake epoch (2010-11-04T01:42:54.657Z).
pub const TWITTER_EPOCH: u64 = 1288834974657;

/// Discord snowflake epoch (2015-01-01T00:00:00.000Z).
pub const DISCORD_EPOCH: u64 = 1420070400000;

/// Snowflake ID generator.
///
/// Thread-safe generator using atomic operations.
pub struct SnowflakeGenerator {
    machine_id: u16,
    epoch: u64,
    /// Packed state: upper 42 bits = timestamp, lower 22 bits = (machine_id << 12) | sequence
    /// Actually we store: upper 42 bits = last_timestamp, lower 12 bits = sequence
    state: AtomicU64,
}

impl SnowflakeGenerator {
    /// Create a new generator with the given machine ID and epoch.
    ///
    /// # Errors
    ///
    /// Returns an error if machine_id > 1023.
    pub fn new(machine_id: u16, epoch: u64) -> Result<Self, SnowflakeError> {
        if machine_id > 1023 {
            return Err(SnowflakeError::InvalidMachineId(machine_id));
        }
        Ok(Self {
            machine_id,
            epoch,
            state: AtomicU64::new(0),
        })
    }

    /// Create a new generator with Twitter's epoch.
    pub fn twitter(machine_id: u16) -> Result<Self, SnowflakeError> {
        Self::new(machine_id, TWITTER_EPOCH)
    }

    /// Create a new generator with Discord's epoch.
    pub fn discord(machine_id: u16) -> Result<Self, SnowflakeError> {
        Self::new(machine_id, DISCORD_EPOCH)
    }

    fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before Unix epoch")
            .as_millis() as u64
            - self.epoch
    }
}

impl Snowflake for SnowflakeGenerator {
    fn next_id(&self) -> Result<SnowflakeId, SnowflakeError> {
        loop {
            let current_ts = self.current_timestamp();
            let old_state = self.state.load(Ordering::Acquire);

            // Extract last timestamp and sequence from state
            // State format: [timestamp (52 bits)][sequence (12 bits)]
            let last_ts = old_state >> 12;
            let last_seq = (old_state & 0xFFF) as u16;

            let (new_ts, new_seq) = if current_ts > last_ts {
                // New millisecond, reset sequence
                (current_ts, 0u16)
            } else if current_ts == last_ts {
                // Same millisecond, increment sequence
                if last_seq >= 4095 {
                    // Sequence exhausted, wait for next millisecond
                    std::hint::spin_loop();
                    continue;
                }
                (current_ts, last_seq + 1)
            } else {
                // Clock moved backwards
                return Err(SnowflakeError::ClockMovedBackwards {
                    last_timestamp: last_ts + self.epoch,
                    current_timestamp: current_ts + self.epoch,
                });
            };

            let new_state = (new_ts << 12) | (new_seq as u64);

            // Try to update state atomically
            if self
                .state
                .compare_exchange(old_state, new_state, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                // Build the snowflake ID
                let id = (new_ts << 22) | ((self.machine_id as u64) << 12) | (new_seq as u64);
                return Ok(SnowflakeId(id));
            }
            // CAS failed, retry
        }
    }

    fn machine_id(&self) -> u16 {
        self.machine_id
    }

    fn epoch(&self) -> u64 {
        self.epoch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_generation() {
        let generator = SnowflakeGenerator::twitter(1).unwrap();
        let id = generator.next_id().unwrap();
        assert_eq!(id.machine_id(), 1);
    }

    #[test]
    fn unique_ids() {
        let generator = SnowflakeGenerator::twitter(1).unwrap();
        let a = generator.next_id().unwrap();
        let b = generator.next_id().unwrap();
        assert_ne!(a, b);
        assert!(b > a); // IDs are monotonically increasing
    }

    #[test]
    fn sequence_increments() {
        let generator = SnowflakeGenerator::twitter(1).unwrap();
        let a = generator.next_id().unwrap();
        let b = generator.next_id().unwrap();
        // In the same millisecond, sequence should increment
        if a.timestamp_bits() == b.timestamp_bits() {
            assert_eq!(b.sequence(), a.sequence() + 1);
        }
    }

    #[test]
    fn machine_id_preserved() {
        let generator = SnowflakeGenerator::twitter(42).unwrap();
        let id = generator.next_id().unwrap();
        assert_eq!(id.machine_id(), 42);
    }

    #[test]
    fn invalid_machine_id() {
        let result = SnowflakeGenerator::twitter(1024);
        assert!(matches!(result, Err(SnowflakeError::InvalidMachineId(1024))));
    }

    #[test]
    fn extract_timestamp() {
        let generator = SnowflakeGenerator::twitter(1).unwrap();
        let id = generator.next_id().unwrap();
        let ts = generator.extract_timestamp(id);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        // Timestamp should be very close to now
        assert!(ts <= now);
        assert!(now - ts < 1000); // Within 1 second
    }

    #[test]
    fn different_epochs() {
        let twitter = SnowflakeGenerator::twitter(1).unwrap();
        let discord = SnowflakeGenerator::discord(1).unwrap();

        let t_id = twitter.next_id().unwrap();
        let d_id = discord.next_id().unwrap();

        // Different epochs should give different timestamp bits for the same moment
        // Discord epoch is later, so its timestamp bits should be smaller
        assert!(t_id.timestamp_bits() > d_id.timestamp_bits());
    }

    #[test]
    fn display() {
        let id = SnowflakeId(123456789);
        assert_eq!(format!("{}", id), "123456789");
    }

    #[test]
    fn conversions() {
        let id = SnowflakeId::from_u64(12345);
        assert_eq!(id.as_u64(), 12345);
        assert_eq!(u64::from(id), 12345);

        let id2: SnowflakeId = 67890u64.into();
        assert_eq!(id2.as_u64(), 67890);
    }
}
