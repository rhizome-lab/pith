//! Key-value store interfaces.
//!
//! Based on WASI key-value.

use std::fmt;
use std::future::Future;

/// Key-value store errors.
#[derive(Debug)]
pub enum Error {
    NotFound,
    Store(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotFound => write!(f, "key not found"),
            Error::Store(msg) => write!(f, "store error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// A key-value store.
pub trait KeyValue {
    /// Get a value by key.
    fn get(&self, key: &str) -> impl Future<Output = Result<Vec<u8>, Error>>;

    /// Set a value.
    fn set(&self, key: &str, value: &[u8]) -> impl Future<Output = Result<(), Error>>;

    /// Delete a key.
    fn delete(&self, key: &str) -> impl Future<Output = Result<(), Error>>;

    /// Check if a key exists.
    fn exists(&self, key: &str) -> impl Future<Output = Result<bool, Error>>;

    /// List all keys.
    fn keys(&self) -> impl Future<Output = Result<Vec<String>, Error>>;
}

/// A key-value store with atomic operations.
pub trait AtomicKeyValue: KeyValue {
    /// Compare and swap - set value only if current value matches expected.
    fn compare_and_swap(
        &self,
        key: &str,
        expected: Option<&[u8]>,
        new: &[u8],
    ) -> impl Future<Output = Result<bool, Error>>;

    /// Increment a numeric value atomically.
    fn increment(&self, key: &str, delta: i64) -> impl Future<Output = Result<i64, Error>>;
}
