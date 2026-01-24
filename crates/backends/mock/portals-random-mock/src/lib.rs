//! Mock implementation of pith-random for testing.
//!
//! Provides deterministic random number generators for reproducible tests.

use rhizome_rhi_portals_random::{InsecureRandom, SecureRandom};
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// A deterministic "secure" random for testing.
///
/// This is NOT cryptographically secure - it's for testing code that
/// requires a `SecureRandom` implementation with reproducible output.
#[derive(Debug, Clone)]
pub struct MockSecureRandom {
    state: Arc<AtomicU64>,
}

impl MockSecureRandom {
    /// Create a new mock random with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: Arc::new(AtomicU64::new(if seed == 0 { 1 } else { seed })),
        }
    }

    fn next(&self) -> u64 {
        loop {
            let current = self.state.load(Ordering::SeqCst);
            let mut x = current;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            if self
                .state
                .compare_exchange(current, x, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return x;
            }
        }
    }
}

impl SecureRandom for MockSecureRandom {
    fn fill(&self, buf: &mut [u8]) {
        for chunk in buf.chunks_mut(8) {
            let val = self.next();
            let bytes = val.to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
    }
}

/// A deterministic insecure random for testing.
///
/// Single-threaded version that's faster than the thread-safe MockSecureRandom.
#[derive(Debug)]
pub struct MockInsecureRandom {
    state: Cell<u64>,
}

impl MockInsecureRandom {
    /// Create a new mock random with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: Cell::new(if seed == 0 { 1 } else { seed }),
        }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state.set(x);
        x
    }
}

impl InsecureRandom for MockInsecureRandom {
    fn fill(&mut self, buf: &mut [u8]) {
        for chunk in buf.chunks_mut(8) {
            let val = self.next();
            let bytes = val.to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_secure_deterministic() {
        let rng1 = MockSecureRandom::new(12345);
        let rng2 = MockSecureRandom::new(12345);

        assert_eq!(rng1.u64(), rng2.u64());
        assert_eq!(rng1.u64(), rng2.u64());
        assert_eq!(rng1.u64(), rng2.u64());
    }

    #[test]
    fn mock_secure_fills_buffer() {
        let rng = MockSecureRandom::new(42);
        let mut buf = [0u8; 32];
        rng.fill(&mut buf);
        assert!(buf.iter().any(|&b| b != 0));
    }

    #[test]
    fn mock_secure_clone_shares_state() {
        let rng1 = MockSecureRandom::new(12345);
        let rng2 = rng1.clone();

        let val1 = rng1.u64();
        let val2 = rng2.u64();
        // They share state, so second call gets the next value
        assert_ne!(val1, val2);
    }

    #[test]
    fn mock_insecure_deterministic() {
        let mut rng1 = MockInsecureRandom::new(12345);
        let mut rng2 = MockInsecureRandom::new(12345);

        assert_eq!(rng1.u64(), rng2.u64());
        assert_eq!(rng1.u64(), rng2.u64());
    }

    #[test]
    fn different_seeds_different_output() {
        let rng1 = MockSecureRandom::new(1);
        let rng2 = MockSecureRandom::new(2);

        assert_ne!(rng1.u64(), rng2.u64());
    }
}
