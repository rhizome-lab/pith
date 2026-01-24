//! Random number generation interfaces.
//!
//! Based on WASI random.

/// Source of cryptographically secure random bytes.
pub trait SecureRandom {
    /// Fill the buffer with random bytes.
    fn fill(&self, buf: &mut [u8]);

    /// Get n random bytes.
    fn bytes(&self, n: usize) -> Vec<u8> {
        let mut buf = vec![0u8; n];
        self.fill(&mut buf);
        buf
    }

    /// Get a random u64.
    fn u64(&self) -> u64 {
        let mut buf = [0u8; 8];
        self.fill(&mut buf);
        u64::from_le_bytes(buf)
    }
}

/// Source of non-cryptographic random bytes (faster, for simulations etc).
///
/// Uses `&mut self` because PRNGs must mutate internal state.
/// See ADR-0001 for rationale.
pub trait InsecureRandom {
    /// Fill the buffer with random bytes.
    fn fill(&mut self, buf: &mut [u8]);

    /// Get a random u64.
    fn u64(&mut self) -> u64 {
        let mut buf = [0u8; 8];
        self.fill(&mut buf);
        u64::from_le_bytes(buf)
    }
}
