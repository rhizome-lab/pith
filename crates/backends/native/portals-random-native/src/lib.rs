//! Native implementation of portals-random.

use portals_random::{InsecureRandom, SecureRandom};

/// Cryptographically secure random using OS entropy.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsRandom;

impl SecureRandom for OsRandom {
    fn fill(&self, buf: &mut [u8]) {
        getrandom::fill(buf).expect("getrandom failed");
    }
}

/// Fast non-cryptographic PRNG (xorshift64).
#[derive(Debug, Clone)]
pub struct FastRandom {
    state: u64,
}

impl FastRandom {
    /// Create a new FastRandom with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    /// Create a new FastRandom seeded from OS entropy.
    pub fn from_entropy() -> Self {
        let mut buf = [0u8; 8];
        getrandom::fill(&mut buf).expect("getrandom failed");
        Self::new(u64::from_le_bytes(buf))
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

impl InsecureRandom for FastRandom {
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
    fn os_random_fills_buffer() {
        let rng = OsRandom;
        let mut buf = [0u8; 32];
        rng.fill(&mut buf);
        assert!(buf.iter().any(|&b| b != 0));
    }

    #[test]
    fn fast_random_deterministic() {
        let mut rng1 = FastRandom::new(12345);
        let mut rng2 = FastRandom::new(12345);
        assert_eq!(rng1.u64(), rng2.u64());
        assert_eq!(rng1.u64(), rng2.u64());
    }

    #[test]
    fn fast_random_fills_buffer() {
        let mut rng = FastRandom::from_entropy();
        let mut buf = [0u8; 32];
        rng.fill(&mut buf);
        assert!(buf.iter().any(|&b| b != 0));
    }
}
