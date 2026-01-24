//! Cache interfaces.
//!
//! In-memory caching with optional TTL (time-to-live) support.

use std::fmt;
use std::time::Duration;

/// A cache for key-value storage with optional TTL.
pub trait Cache {
    /// Get a value by key.
    ///
    /// Returns `None` if the key doesn't exist or has expired.
    fn get(&self, key: &str) -> Option<Vec<u8>>;

    /// Set a value with no expiration.
    fn set(&self, key: &str, value: Vec<u8>);

    /// Set a value with a TTL.
    ///
    /// The value will be automatically removed after the TTL expires.
    fn set_with_ttl(&self, key: &str, value: Vec<u8>, ttl: Duration);

    /// Delete a key.
    ///
    /// Returns `true` if the key existed.
    fn delete(&self, key: &str) -> bool;

    /// Check if a key exists (and hasn't expired).
    fn exists(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Clear all entries.
    fn clear(&self);
}

/// A typed cache wrapper.
pub trait TypedCache<T> {
    /// Get a value by key.
    fn get(&self, key: &str) -> Option<T>;

    /// Set a value with no expiration.
    fn set(&self, key: &str, value: T);

    /// Set a value with a TTL.
    fn set_with_ttl(&self, key: &str, value: T, ttl: Duration);

    /// Delete a key.
    fn delete(&self, key: &str) -> bool;

    /// Check if a key exists.
    fn exists(&self, key: &str) -> bool;

    /// Clear all entries.
    fn clear(&self);
}

/// Cache entry with metadata.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The cached value.
    pub value: Vec<u8>,
    /// When the entry was created.
    pub created_at: Duration,
    /// Time-to-live (if set).
    pub ttl: Option<Duration>,
}

impl CacheEntry {
    /// Check if this entry has expired.
    pub fn is_expired(&self, now: Duration) -> bool {
        if let Some(ttl) = self.ttl {
            now > self.created_at + ttl
        } else {
            false
        }
    }

    /// Get remaining TTL (if any).
    pub fn remaining_ttl(&self, now: Duration) -> Option<Duration> {
        self.ttl.and_then(|ttl| {
            let expires_at = self.created_at + ttl;
            if now < expires_at {
                Some(expires_at - now)
            } else {
                None // Already expired
            }
        })
    }
}

/// Error type for cache operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheError {
    /// Key not found.
    NotFound,
    /// Value too large.
    ValueTooLarge { max_size: usize, actual_size: usize },
    /// Cache is full.
    CacheFull,
    /// Serialization error.
    SerializationError(String),
    /// Other error.
    Other(String),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "key not found"),
            Self::ValueTooLarge {
                max_size,
                actual_size,
            } => {
                write!(
                    f,
                    "value too large: {} bytes (max {})",
                    actual_size, max_size
                )
            }
            Self::CacheFull => write!(f, "cache is full"),
            Self::SerializationError(msg) => write!(f, "serialization error: {}", msg),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CacheError {}

/// Cache statistics.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits.
    pub hits: u64,
    /// Number of cache misses.
    pub misses: u64,
    /// Number of entries currently in cache.
    pub entries: usize,
    /// Total size of cached values in bytes.
    pub size_bytes: usize,
}

impl CacheStats {
    /// Calculate hit rate (0.0 to 1.0).
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// A cache that tracks statistics.
pub trait CacheWithStats: Cache {
    /// Get cache statistics.
    fn stats(&self) -> CacheStats;

    /// Reset statistics.
    fn reset_stats(&self);
}
