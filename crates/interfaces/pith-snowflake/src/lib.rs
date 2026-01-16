//! Snowflake ID interfaces.
//!
//! Twitter-style snowflake IDs: 64-bit unique identifiers that encode
//! timestamp, machine ID, and sequence number.

use std::fmt;

/// A snowflake ID (64-bit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SnowflakeId(pub u64);

impl SnowflakeId {
    /// Create from raw u64 value.
    pub fn from_u64(value: u64) -> Self {
        Self(value)
    }

    /// Get the raw u64 value.
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Extract timestamp (milliseconds since epoch).
    ///
    /// The epoch is configurable per generator, so this returns the raw
    /// timestamp bits. Use the generator's `extract_timestamp` for absolute time.
    pub fn timestamp_bits(&self) -> u64 {
        self.0 >> 22
    }

    /// Extract machine/worker ID (10 bits, 0-1023).
    pub fn machine_id(&self) -> u16 {
        ((self.0 >> 12) & 0x3FF) as u16
    }

    /// Extract sequence number (12 bits, 0-4095).
    pub fn sequence(&self) -> u16 {
        (self.0 & 0xFFF) as u16
    }
}

impl fmt::Display for SnowflakeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for SnowflakeId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<SnowflakeId> for u64 {
    fn from(id: SnowflakeId) -> Self {
        id.0
    }
}

/// Error generating snowflake ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnowflakeError {
    /// Clock moved backwards.
    ClockMovedBackwards {
        /// Last timestamp seen.
        last_timestamp: u64,
        /// Current timestamp.
        current_timestamp: u64,
    },
    /// Sequence exhausted for this millisecond.
    SequenceExhausted,
    /// Invalid machine ID (must be 0-1023).
    InvalidMachineId(u16),
    /// Other error.
    Other(String),
}

impl fmt::Display for SnowflakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClockMovedBackwards {
                last_timestamp,
                current_timestamp,
            } => write!(
                f,
                "clock moved backwards: last={}, current={}",
                last_timestamp, current_timestamp
            ),
            Self::SequenceExhausted => write!(f, "sequence exhausted for this millisecond"),
            Self::InvalidMachineId(id) => write!(f, "invalid machine ID: {} (must be 0-1023)", id),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for SnowflakeError {}

/// Generator for snowflake IDs.
pub trait Snowflake {
    /// Generate the next snowflake ID.
    ///
    /// Returns an error if the clock moved backwards or sequence is exhausted.
    fn next_id(&self) -> Result<SnowflakeId, SnowflakeError>;

    /// Get the machine ID this generator uses.
    fn machine_id(&self) -> u16;

    /// Get the epoch (milliseconds since Unix epoch) this generator uses.
    fn epoch(&self) -> u64;

    /// Extract absolute timestamp from an ID (milliseconds since Unix epoch).
    fn extract_timestamp(&self, id: SnowflakeId) -> u64 {
        id.timestamp_bits() + self.epoch()
    }
}
