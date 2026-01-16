//! Native in-memory cache implementation.

use rhizome_pith_cache::{Cache, CacheEntry, CacheStats, CacheWithStats};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Thread-safe in-memory cache.
pub struct MemoryCache {
    entries: RwLock<HashMap<String, Entry>>,
    start_time: Instant,
    hits: AtomicU64,
    misses: AtomicU64,
}

struct Entry {
    value: Vec<u8>,
    created_at: Duration,
    ttl: Option<Duration>,
}

impl Entry {
    fn is_expired(&self, now: Duration) -> bool {
        if let Some(ttl) = self.ttl {
            now > self.created_at + ttl
        } else {
            false
        }
    }

    fn to_cache_entry(&self) -> CacheEntry {
        CacheEntry {
            value: self.value.clone(),
            created_at: self.created_at,
            ttl: self.ttl,
        }
    }
}

impl MemoryCache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            start_time: Instant::now(),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    /// Get the current time since cache creation.
    fn now(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get entry with metadata.
    pub fn get_entry(&self, key: &str) -> Option<CacheEntry> {
        let now = self.now();
        let entries = self.entries.read().unwrap();

        if let Some(entry) = entries.get(key) {
            if entry.is_expired(now) {
                drop(entries);
                // Remove expired entry
                self.entries.write().unwrap().remove(key);
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            } else {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(entry.to_cache_entry())
            }
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Remove expired entries.
    pub fn cleanup(&self) {
        let now = self.now();
        let mut entries = self.entries.write().unwrap();
        entries.retain(|_, entry| !entry.is_expired(now));
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache for MemoryCache {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.get_entry(key).map(|e| e.value)
    }

    fn set(&self, key: &str, value: Vec<u8>) {
        let now = self.now();
        let mut entries = self.entries.write().unwrap();
        entries.insert(
            key.to_string(),
            Entry {
                value,
                created_at: now,
                ttl: None,
            },
        );
    }

    fn set_with_ttl(&self, key: &str, value: Vec<u8>, ttl: Duration) {
        let now = self.now();
        let mut entries = self.entries.write().unwrap();
        entries.insert(
            key.to_string(),
            Entry {
                value,
                created_at: now,
                ttl: Some(ttl),
            },
        );
    }

    fn delete(&self, key: &str) -> bool {
        self.entries.write().unwrap().remove(key).is_some()
    }

    fn exists(&self, key: &str) -> bool {
        let now = self.now();
        let entries = self.entries.read().unwrap();

        if let Some(entry) = entries.get(key) {
            if entry.is_expired(now) {
                drop(entries);
                self.entries.write().unwrap().remove(key);
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    fn clear(&self) {
        self.entries.write().unwrap().clear();
    }
}

impl CacheWithStats for MemoryCache {
    fn stats(&self) -> CacheStats {
        let entries = self.entries.read().unwrap();
        let size_bytes: usize = entries.values().map(|e| e.value.len()).sum();

        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            entries: entries.len(),
            size_bytes,
        }
    }

    fn reset_stats(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn basic_set_get() {
        let cache = MemoryCache::new();
        cache.set("key", b"value".to_vec());
        assert_eq!(cache.get("key"), Some(b"value".to_vec()));
    }

    #[test]
    fn get_nonexistent() {
        let cache = MemoryCache::new();
        assert_eq!(cache.get("missing"), None);
    }

    #[test]
    fn delete() {
        let cache = MemoryCache::new();
        cache.set("key", b"value".to_vec());
        assert!(cache.delete("key"));
        assert_eq!(cache.get("key"), None);
        assert!(!cache.delete("key"));
    }

    #[test]
    fn exists() {
        let cache = MemoryCache::new();
        assert!(!cache.exists("key"));
        cache.set("key", b"value".to_vec());
        assert!(cache.exists("key"));
    }

    #[test]
    fn clear() {
        let cache = MemoryCache::new();
        cache.set("a", b"1".to_vec());
        cache.set("b", b"2".to_vec());
        cache.clear();
        assert!(!cache.exists("a"));
        assert!(!cache.exists("b"));
    }

    #[test]
    fn ttl_expiration() {
        let cache = MemoryCache::new();
        cache.set_with_ttl("key", b"value".to_vec(), Duration::from_millis(50));

        // Should exist immediately
        assert!(cache.exists("key"));

        // Wait for expiration
        thread::sleep(Duration::from_millis(100));

        // Should be gone
        assert!(!cache.exists("key"));
        assert_eq!(cache.get("key"), None);
    }

    #[test]
    fn stats() {
        let cache = MemoryCache::new();

        cache.set("key", b"value".to_vec());
        let _ = cache.get("key"); // hit
        let _ = cache.get("key"); // hit
        let _ = cache.get("missing"); // miss

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.entries, 1);
        assert_eq!(stats.size_bytes, 5);

        assert!((stats.hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn reset_stats() {
        let cache = MemoryCache::new();
        cache.set("key", b"value".to_vec());
        let _ = cache.get("key");
        let _ = cache.get("missing");

        cache.reset_stats();
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        // entries and size should still be there
        assert_eq!(stats.entries, 1);
    }

    #[test]
    fn overwrite() {
        let cache = MemoryCache::new();
        cache.set("key", b"old".to_vec());
        cache.set("key", b"new".to_vec());
        assert_eq!(cache.get("key"), Some(b"new".to_vec()));
    }

    #[test]
    fn cleanup() {
        let cache = MemoryCache::new();
        cache.set_with_ttl("a", b"1".to_vec(), Duration::from_millis(10));
        cache.set("b", b"2".to_vec());

        thread::sleep(Duration::from_millis(50));
        cache.cleanup();

        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        assert!(!cache.exists("a"));
        assert!(cache.exists("b"));
    }

    #[test]
    fn thread_safety() {
        use std::sync::Arc;

        let cache = Arc::new(MemoryCache::new());
        let mut handles = vec![];

        for i in 0..10 {
            let cache = Arc::clone(&cache);
            handles.push(thread::spawn(move || {
                let key = format!("key{}", i);
                cache.set(&key, vec![i as u8]);
                assert_eq!(cache.get(&key), Some(vec![i as u8]));
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = cache.stats();
        assert_eq!(stats.entries, 10);
    }
}
