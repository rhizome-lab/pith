//! Runtime configuration interfaces.
//!
//! Based on WASI runtime-config.

/// Configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("key not found: {0}")]
    NotFound(String),
    #[error("invalid value: {0}")]
    InvalidValue(String),
    #[error("{0}")]
    Other(String),
}

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
