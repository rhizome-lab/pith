//! Encoding/decoding interfaces.

/// Base64 encoding/decoding.
pub trait Base64 {
    /// Encode bytes to base64 string.
    fn encode(data: &[u8]) -> String;

    /// Decode base64 string to bytes.
    fn decode(encoded: &str) -> Result<Vec<u8>, DecodeError>;
}

/// URL-safe Base64 encoding/decoding (no padding).
pub trait Base64Url {
    /// Encode bytes to URL-safe base64 string.
    fn encode(data: &[u8]) -> String;

    /// Decode URL-safe base64 string to bytes.
    fn decode(encoded: &str) -> Result<Vec<u8>, DecodeError>;
}

/// Hexadecimal encoding/decoding.
pub trait Hex {
    /// Encode bytes to hex string.
    fn encode(data: &[u8]) -> String;

    /// Encode bytes to uppercase hex string.
    fn encode_upper(data: &[u8]) -> String;

    /// Decode hex string to bytes.
    fn decode(encoded: &str) -> Result<Vec<u8>, DecodeError>;
}

/// URL encoding/decoding (percent encoding).
pub trait UrlEncoding {
    /// Encode a string for use in URLs.
    fn encode(input: &str) -> String;

    /// Decode a URL-encoded string.
    fn decode(encoded: &str) -> Result<String, DecodeError>;
}

/// Decoding errors.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    /// Invalid character in input.
    #[error("invalid character: {0:?}")]
    InvalidCharacter(char),
    /// Invalid length.
    #[error("invalid length")]
    InvalidLength,
    /// Invalid padding.
    #[error("invalid padding")]
    InvalidPadding,
    /// Invalid UTF-8.
    #[error("invalid UTF-8")]
    InvalidUtf8,
    /// Other error.
    #[error("{0}")]
    Other(String),
}
