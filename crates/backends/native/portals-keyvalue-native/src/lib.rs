//! Native key-value store implementation.

use portals_keyvalue::{AtomicKeyValue, Error, KeyValue};
use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory key-value store.
#[derive(Debug, Default)]
pub struct MemoryStore {
    data: RwLock<HashMap<String, Vec<u8>>>,
}

impl MemoryStore {
    /// Create a new empty store.
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

impl KeyValue for MemoryStore {
    async fn get(&self, key: &str) -> Result<Vec<u8>, Error> {
        let data = self.data.read().map_err(|e| Error::Store(e.to_string()))?;
        data.get(key).cloned().ok_or(Error::NotFound)
    }

    async fn set(&self, key: &str, value: &[u8]) -> Result<(), Error> {
        let mut data = self.data.write().map_err(|e| Error::Store(e.to_string()))?;
        data.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), Error> {
        let mut data = self.data.write().map_err(|e| Error::Store(e.to_string()))?;
        data.remove(key).ok_or(Error::NotFound)?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, Error> {
        let data = self.data.read().map_err(|e| Error::Store(e.to_string()))?;
        Ok(data.contains_key(key))
    }

    async fn keys(&self) -> Result<Vec<String>, Error> {
        let data = self.data.read().map_err(|e| Error::Store(e.to_string()))?;
        Ok(data.keys().cloned().collect())
    }
}

impl AtomicKeyValue for MemoryStore {
    async fn compare_and_swap(
        &self,
        key: &str,
        expected: Option<&[u8]>,
        new: &[u8],
    ) -> Result<bool, Error> {
        let mut data = self.data.write().map_err(|e| Error::Store(e.to_string()))?;
        let current = data.get(key);

        let matches = match (expected, current) {
            (None, None) => true,
            (Some(exp), Some(cur)) => exp == cur.as_slice(),
            _ => false,
        };

        if matches {
            data.insert(key.to_string(), new.to_vec());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn increment(&self, key: &str, delta: i64) -> Result<i64, Error> {
        let mut data = self.data.write().map_err(|e| Error::Store(e.to_string()))?;

        let current = data
            .get(key)
            .map(|v| {
                let arr: [u8; 8] = v.as_slice().try_into().unwrap_or([0; 8]);
                i64::from_le_bytes(arr)
            })
            .unwrap_or(0);

        let new_value = current + delta;
        data.insert(key.to_string(), new_value.to_le_bytes().to_vec());
        Ok(new_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic_operations() {
        let store = MemoryStore::new();

        store.set("key", b"value").await.unwrap();
        assert_eq!(store.get("key").await.unwrap(), b"value");
        assert!(store.exists("key").await.unwrap());

        store.delete("key").await.unwrap();
        assert!(!store.exists("key").await.unwrap());
    }

    #[tokio::test]
    async fn keys_list() {
        let store = MemoryStore::new();

        store.set("a", b"1").await.unwrap();
        store.set("b", b"2").await.unwrap();

        let keys = store.keys().await.unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn compare_and_swap() {
        let store = MemoryStore::new();

        // Set if not exists
        assert!(store.compare_and_swap("key", None, b"value").await.unwrap());

        // Should fail - key exists now
        assert!(!store.compare_and_swap("key", None, b"other").await.unwrap());

        // Should succeed - matches current value
        assert!(store
            .compare_and_swap("key", Some(b"value"), b"new")
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn increment() {
        let store = MemoryStore::new();

        assert_eq!(store.increment("counter", 1).await.unwrap(), 1);
        assert_eq!(store.increment("counter", 5).await.unwrap(), 6);
        assert_eq!(store.increment("counter", -2).await.unwrap(), 4);
    }
}
