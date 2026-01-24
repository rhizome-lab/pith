//! Native configuration implementation using environment variables.

use portals_config::{Config, ConfigMut, Error};
use std::collections::HashMap;
use std::env;

/// Configuration from environment variables.
#[derive(Debug, Default)]
pub struct EnvConfig {
    prefix: Option<String>,
}

impl EnvConfig {
    /// Create a new environment config.
    pub fn new() -> Self {
        Self { prefix: None }
    }

    /// Create a new environment config with a prefix.
    /// Keys will be looked up as `{PREFIX}_{KEY}`.
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: Some(prefix.into()),
        }
    }

    fn make_key(&self, key: &str) -> String {
        match &self.prefix {
            Some(prefix) => format!("{}_{}", prefix, key),
            None => key.to_string(),
        }
    }
}

impl Config for EnvConfig {
    fn get(&self, key: &str) -> Result<String, Error> {
        let env_key = self.make_key(key);
        env::var(&env_key).map_err(|_| Error::NotFound(key.to_string()))
    }

    fn keys(&self) -> Vec<String> {
        let vars: Vec<String> = env::vars()
            .filter_map(|(k, _)| match &self.prefix {
                Some(prefix) => {
                    if k.starts_with(prefix) {
                        Some(k.strip_prefix(&format!("{}_", prefix))?.to_string())
                    } else {
                        None
                    }
                }
                None => Some(k),
            })
            .collect();
        vars
    }
}

/// In-memory configuration.
#[derive(Debug, Default, Clone)]
pub struct MemoryConfig {
    values: HashMap<String, String>,
}

impl MemoryConfig {
    /// Create a new empty memory config.
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Create from a list of key-value pairs.
    pub fn from_pairs(pairs: impl IntoIterator<Item = (String, String)>) -> Self {
        Self {
            values: pairs.into_iter().collect(),
        }
    }
}

impl Config for MemoryConfig {
    fn get(&self, key: &str) -> Result<String, Error> {
        self.values
            .get(key)
            .cloned()
            .ok_or_else(|| Error::NotFound(key.to_string()))
    }

    fn keys(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }
}

impl ConfigMut for MemoryConfig {
    fn set(&mut self, key: &str, value: &str) -> Result<(), Error> {
        self.values.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn remove(&mut self, key: &str) -> Result<(), Error> {
        self.values
            .remove(key)
            .map(|_| ())
            .ok_or_else(|| Error::NotFound(key.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_config_reads_path() {
        let config = EnvConfig::new();
        // PATH should exist on most systems
        assert!(config.get("PATH").is_ok());
    }

    #[test]
    fn env_config_not_found() {
        let config = EnvConfig::new();
        assert!(config.get("DEFINITELY_NOT_A_REAL_VAR_12345").is_err());
    }

    #[test]
    fn memory_config_basic() {
        let mut config = MemoryConfig::new();
        config.set("key", "value").unwrap();
        assert_eq!(config.get("key").unwrap(), "value");
    }

    #[test]
    fn memory_config_remove() {
        let mut config = MemoryConfig::new();
        config.set("key", "value").unwrap();
        config.remove("key").unwrap();
        assert!(config.get("key").is_err());
    }
}
