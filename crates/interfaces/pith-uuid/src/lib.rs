//! UUID interfaces.
//!
//! Based on RFC 4122 and RFC 9562 (UUIDv7).

use std::fmt;
use std::str::FromStr;

/// A universally unique identifier.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid([u8; 16]);

impl Uuid {
    /// Create a UUID from bytes.
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Get the UUID as bytes.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Get the UUID version.
    pub fn version(&self) -> u8 {
        (self.0[6] >> 4) & 0x0f
    }

    /// Check if this is a nil (all zeros) UUID.
    pub fn is_nil(&self) -> bool {
        self.0 == [0u8; 16]
    }

    /// The nil UUID (all zeros).
    pub const NIL: Self = Self([0u8; 16]);
}

impl fmt::Debug for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Uuid({})", self)
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3],
            self.0[4], self.0[5],
            self.0[6], self.0[7],
            self.0[8], self.0[9],
            self.0[10], self.0[11], self.0[12], self.0[13], self.0[14], self.0[15]
        )
    }
}

/// UUID parsing error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid UUID")
    }
}

impl std::error::Error for ParseError {}

impl FromStr for Uuid {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Accept with or without hyphens
        let s = s.trim();
        let bytes: Vec<u8> = if s.len() == 36 {
            // xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
            s.split('-')
                .flat_map(|part| {
                    (0..part.len())
                        .step_by(2)
                        .map(|i| u8::from_str_radix(&part[i..i + 2], 16).ok())
                })
                .collect::<Option<Vec<_>>>()
                .ok_or(ParseError)?
        } else if s.len() == 32 {
            // xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
            (0..32)
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
                .collect::<Option<Vec<_>>>()
                .ok_or(ParseError)?
        } else {
            return Err(ParseError);
        };

        if bytes.len() != 16 {
            return Err(ParseError);
        }

        let mut arr = [0u8; 16];
        arr.copy_from_slice(&bytes);
        Ok(Uuid(arr))
    }
}

/// Generator for random UUIDs (v4).
pub trait UuidV4 {
    /// Generate a random UUID v4.
    fn v4(&self) -> Uuid;
}

/// Generator for timestamp-based UUIDs (v7).
pub trait UuidV7 {
    /// Generate a timestamp-based UUID v7.
    fn v7(&self) -> Uuid;
}
