//! Nanoid interfaces.
//!
//! Tiny, secure, URL-friendly unique string IDs.

/// Default alphabet (URL-safe).
pub const DEFAULT_ALPHABET: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz-";

/// Default ID length.
pub const DEFAULT_LENGTH: usize = 21;

/// Generator for nanoid-style IDs.
pub trait NanoId {
    /// Generate a nanoid with default settings (21 chars, URL-safe alphabet).
    fn nanoid(&self) -> String;

    /// Generate a nanoid with custom length.
    fn nanoid_len(&self, len: usize) -> String;

    /// Generate a nanoid with custom alphabet and length.
    fn nanoid_custom(&self, len: usize, alphabet: &str) -> String;
}
