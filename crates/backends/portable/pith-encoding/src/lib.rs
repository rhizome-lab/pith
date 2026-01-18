//! Portable implementation of pith-encoding.
//!
//! Works on both native and WASM targets.

use rhizome_pith_encoding::{Base64, Base64Url, DecodeError, Hex, UrlEncoding};

/// Standard Base64 encoding.
pub struct StdBase64;

impl Base64 for StdBase64 {
    fn encode(data: &[u8]) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(data)
    }

    fn decode(encoded: &str) -> Result<Vec<u8>, DecodeError> {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|_| DecodeError::InvalidCharacter('?'))
    }
}

/// URL-safe Base64 encoding (no padding).
pub struct StdBase64Url;

impl Base64Url for StdBase64Url {
    fn encode(data: &[u8]) -> String {
        use base64::Engine;
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
    }

    fn decode(encoded: &str) -> Result<Vec<u8>, DecodeError> {
        use base64::Engine;
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| DecodeError::InvalidCharacter('?'))
    }
}

/// Hexadecimal encoding.
pub struct StdHex;

impl Hex for StdHex {
    fn encode(data: &[u8]) -> String {
        data.iter().map(|b| format!("{:02x}", b)).collect()
    }

    fn encode_upper(data: &[u8]) -> String {
        data.iter().map(|b| format!("{:02X}", b)).collect()
    }

    fn decode(encoded: &str) -> Result<Vec<u8>, DecodeError> {
        if encoded.len() % 2 != 0 {
            return Err(DecodeError::InvalidLength);
        }

        let mut result = Vec::with_capacity(encoded.len() / 2);
        let mut chars = encoded.chars();

        while let (Some(hi), Some(lo)) = (chars.next(), chars.next()) {
            let hi = hi
                .to_digit(16)
                .ok_or(DecodeError::InvalidCharacter(hi))? as u8;
            let lo = lo
                .to_digit(16)
                .ok_or(DecodeError::InvalidCharacter(lo))? as u8;
            result.push((hi << 4) | lo);
        }

        Ok(result)
    }
}

/// URL percent encoding.
pub struct StdUrlEncoding;

impl UrlEncoding for StdUrlEncoding {
    fn encode(input: &str) -> String {
        let mut result = String::new();
        for c in input.chars() {
            match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                    result.push(c);
                }
                _ => {
                    for b in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", b));
                    }
                }
            }
        }
        result
    }

    fn decode(encoded: &str) -> Result<String, DecodeError> {
        let mut result = Vec::new();
        let mut chars = encoded.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '%' {
                let hi = chars.next().ok_or(DecodeError::InvalidLength)?;
                let lo = chars.next().ok_or(DecodeError::InvalidLength)?;
                let hi = hi
                    .to_digit(16)
                    .ok_or(DecodeError::InvalidCharacter(hi))? as u8;
                let lo = lo
                    .to_digit(16)
                    .ok_or(DecodeError::InvalidCharacter(lo))? as u8;
                result.push((hi << 4) | lo);
            } else if c == '+' {
                result.push(b' ');
            } else {
                result.push(c as u8);
            }
        }

        String::from_utf8(result).map_err(|_| DecodeError::InvalidUtf8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_roundtrip() {
        let data = b"hello world";
        let encoded = StdBase64::encode(data);
        assert_eq!(encoded, "aGVsbG8gd29ybGQ=");
        let decoded = StdBase64::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn base64url_roundtrip() {
        let data = b"hello world";
        let encoded = StdBase64Url::encode(data);
        assert_eq!(encoded, "aGVsbG8gd29ybGQ");
        let decoded = StdBase64Url::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn hex_roundtrip() {
        let data = b"\xde\xad\xbe\xef";
        let encoded = StdHex::encode(data);
        assert_eq!(encoded, "deadbeef");
        let encoded_upper = StdHex::encode_upper(data);
        assert_eq!(encoded_upper, "DEADBEEF");
        let decoded = StdHex::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn url_encoding_roundtrip() {
        let input = "hello world!";
        let encoded = StdUrlEncoding::encode(input);
        assert_eq!(encoded, "hello%20world%21");
        let decoded = StdUrlEncoding::decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }
}
