//! Native UUID implementation using the uuid crate.

use pith_uuid::{Uuid, UuidV4, UuidV7};

/// UUID generator using the uuid crate.
#[derive(Debug, Default, Clone, Copy)]
pub struct UuidGenerator;

impl UuidGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl UuidV4 for UuidGenerator {
    fn v4(&self) -> Uuid {
        let uuid = uuid::Uuid::new_v4();
        Uuid::from_bytes(*uuid.as_bytes())
    }
}

impl UuidV7 for UuidGenerator {
    fn v7(&self) -> Uuid {
        let uuid = uuid::Uuid::now_v7();
        Uuid::from_bytes(*uuid.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn generate_v4() {
        let generator = UuidGenerator::new();
        let uuid = generator.v4();
        assert_eq!(uuid.version(), 4);
        assert!(!uuid.is_nil());
    }

    #[test]
    fn generate_v7() {
        let generator = UuidGenerator::new();
        let uuid = generator.v7();
        assert_eq!(uuid.version(), 7);
        assert!(!uuid.is_nil());
    }

    #[test]
    fn v7_is_sortable() {
        let generator = UuidGenerator::new();
        let a = generator.v7();
        let b = generator.v7();
        // Later UUIDs should be >= earlier ones (by bytes)
        assert!(a.as_bytes() <= b.as_bytes());
    }

    #[test]
    fn format_and_parse() {
        let generator = UuidGenerator::new();
        let uuid = generator.v4();
        let s = uuid.to_string();
        let parsed = Uuid::from_str(&s).unwrap();
        assert_eq!(uuid, parsed);
    }

    #[test]
    fn parse_without_hyphens() {
        let uuid = Uuid::from_str("550e8400e29b41d4a716446655440000").unwrap();
        assert_eq!(uuid.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }
}
