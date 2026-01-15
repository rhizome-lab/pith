//! URL parsing and manipulation.
//!
//! Wraps the `url` crate as a blessed default.

pub use url::{Host, ParseError, Position, Url};

/// Parse a URL string.
pub fn parse(input: &str) -> Result<Url, ParseError> {
    Url::parse(input)
}

/// Parse a URL string with a base URL for relative resolution.
pub fn parse_with_base(base: &Url, input: &str) -> Result<Url, ParseError> {
    base.join(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_url() {
        let url = parse("https://example.com/path?query=1#fragment").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.path(), "/path");
        assert_eq!(url.query(), Some("query=1"));
        assert_eq!(url.fragment(), Some("fragment"));
    }

    #[test]
    fn parse_with_port() {
        let url = parse("http://localhost:8080/api").unwrap();
        assert_eq!(url.host_str(), Some("localhost"));
        assert_eq!(url.port(), Some(8080));
    }

    #[test]
    fn relative_url_resolution() {
        let base = parse("https://example.com/a/b/c").unwrap();
        let resolved = parse_with_base(&base, "../d").unwrap();
        assert_eq!(resolved.path(), "/a/d");
    }

    #[test]
    fn invalid_url_fails() {
        assert!(parse("not a url").is_err());
    }
}
