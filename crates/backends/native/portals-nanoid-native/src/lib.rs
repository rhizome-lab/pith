//! Native nanoid implementation using the nanoid crate.

use portals_nanoid::NanoId;

/// Nanoid generator using the nanoid crate.
#[derive(Debug, Default, Clone, Copy)]
pub struct NanoIdGenerator;

impl NanoIdGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl NanoId for NanoIdGenerator {
    fn nanoid(&self) -> String {
        nanoid::nanoid!()
    }

    fn nanoid_len(&self, len: usize) -> String {
        nanoid::nanoid!(len)
    }

    fn nanoid_custom(&self, len: usize, alphabet: &str) -> String {
        let alphabet: Vec<char> = alphabet.chars().collect();
        nanoid::nanoid!(len, &alphabet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_length() {
        let generator = NanoIdGenerator::new();
        let id = generator.nanoid();
        assert_eq!(id.len(), 21);
    }

    #[test]
    fn custom_length() {
        let generator = NanoIdGenerator::new();
        let id = generator.nanoid_len(10);
        assert_eq!(id.len(), 10);
    }

    #[test]
    fn custom_alphabet() {
        let generator = NanoIdGenerator::new();
        let id = generator.nanoid_custom(8, "abc123");
        assert_eq!(id.len(), 8);
        assert!(id.chars().all(|c| "abc123".contains(c)));
    }

    #[test]
    fn unique_ids() {
        let generator = NanoIdGenerator::new();
        let a = generator.nanoid();
        let b = generator.nanoid();
        assert_ne!(a, b);
    }
}
