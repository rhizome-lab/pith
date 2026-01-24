//! Runtime configuration interfaces.
//!
//! Based on WASI runtime-config.

use std::fmt;

/// Configuration errors.
#[derive(Debug)]
pub enum Error {
    NotFound(String),
    InvalidValue(String),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotFound(key) => write!(f, "key not found: {}", key),
            Error::InvalidValue(msg) => write!(f, "invalid value: {}", msg),
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// A configuration source.
pub trait Config {
    /// Get a configuration value by key.
    fn get(&self, key: &str) -> Result<String, Error>;

    /// Get a configuration value, returning None if not found.
    fn get_optional(&self, key: &str) -> Option<String> {
        self.get(key).ok()
    }

    /// Get all configuration keys.
    fn keys(&self) -> Vec<String>;
}

/// A mutable configuration source.
pub trait ConfigMut: Config {
    /// Set a configuration value.
    fn set(&mut self, key: &str, value: &str) -> Result<(), Error>;

    /// Remove a configuration value.
    fn remove(&mut self, key: &str) -> Result<(), Error>;
}
