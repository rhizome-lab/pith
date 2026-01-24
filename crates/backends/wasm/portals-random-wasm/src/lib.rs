//! WASM implementation of portals-random.
//!
//! Uses the Web Crypto API via `getrandom` crate with `wasm_js` feature.

use portals_random::{InsecureRandom, SecureRandom};

/// Cryptographically secure random using Web Crypto API.
#[derive(Debug, Default, Clone, Copy)]
pub struct WebCryptoRandom;

impl SecureRandom for WebCryptoRandom {
    fn fill(&self, buf: &mut [u8]) {
        getrandom::fill(buf).expect("Web Crypto API should be available");
    }
}

/// Non-cryptographic PRNG using xoshiro256++.
///
/// Seeded from Web Crypto API on creation.
#[derive(Debug, Clone)]
pub struct Xoshiro256PlusPlus {
    state: [u64; 4],
}

impl Default for Xoshiro256PlusPlus {
    fn default() -> Self {
        Self::new()
    }
}

impl Xoshiro256PlusPlus {
    /// Create a new PRNG seeded from Web Crypto API.
    pub fn new() -> Self {
        let mut seed = [0u8; 32];
        getrandom::fill(&mut seed).expect("Web Crypto API should be available");
        Self::from_seed(seed)
    }

    /// Create a new PRNG from a seed.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let state = [
            u64::from_le_bytes(seed[0..8].try_into().unwrap()),
            u64::from_le_bytes(seed[8..16].try_into().unwrap()),
            u64::from_le_bytes(seed[16..24].try_into().unwrap()),
            u64::from_le_bytes(seed[24..32].try_into().unwrap()),
        ];
        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        let result = self.state[0]
            .wrapping_add(self.state[3])
            .rotate_left(23)
            .wrapping_add(self.state[0]);

        let t = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);

        result
    }
}

impl InsecureRandom for Xoshiro256PlusPlus {
    fn fill(&mut self, buf: &mut [u8]) {
        let mut i = 0;
        while i < buf.len() {
            let random = self.next_u64();
            let bytes = random.to_le_bytes();
            let remaining = buf.len() - i;
            let to_copy = remaining.min(8);
            buf[i..i + to_copy].copy_from_slice(&bytes[..to_copy]);
            i += to_copy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn secure_random_fills_buffer() {
        let rng = WebCryptoRandom;
        let mut buf = [0u8; 32];
        rng.fill(&mut buf);
        // Very unlikely to be all zeros
        assert!(buf.iter().any(|&b| b != 0));
    }

    #[wasm_bindgen_test]
    fn secure_random_different_each_time() {
        let rng = WebCryptoRandom;
        let mut buf1 = [0u8; 32];
        let mut buf2 = [0u8; 32];
        rng.fill(&mut buf1);
        rng.fill(&mut buf2);
        assert_ne!(buf1, buf2);
    }

    #[wasm_bindgen_test]
    fn insecure_random_deterministic() {
        let seed = [42u8; 32];
        let mut rng1 = Xoshiro256PlusPlus::from_seed(seed);
        let mut rng2 = Xoshiro256PlusPlus::from_seed(seed);

        let mut buf1 = [0u8; 16];
        let mut buf2 = [0u8; 16];
        rng1.fill(&mut buf1);
        rng2.fill(&mut buf2);
        assert_eq!(buf1, buf2);
    }
}
