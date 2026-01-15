//! Timezone handling.
//!
//! Wraps the `jiff` crate for timezone operations.

pub use jiff::tz::{TimeZone, TimeZoneDatabase};
pub use jiff::{Timestamp, Zoned};

/// Timezone errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid timezone: {0}")]
    InvalidTimezone(String),
    #[error("conversion error: {0}")]
    Conversion(String),
}

/// Get the system's local timezone.
pub fn local() -> TimeZone {
    TimeZone::system()
}

/// Get a timezone by IANA name (e.g., "America/New_York").
pub fn get(name: &str) -> Result<TimeZone, Error> {
    TimeZone::get(name).map_err(|e| Error::InvalidTimezone(e.to_string()))
}

/// Get the UTC timezone.
pub fn utc() -> TimeZone {
    TimeZone::UTC
}

/// Get the current time in a specific timezone.
pub fn now_in(tz: &TimeZone) -> Zoned {
    Zoned::now().with_time_zone(tz.clone())
}

/// Convert a timestamp to a specific timezone.
pub fn to_timezone(timestamp: Timestamp, tz: &TimeZone) -> Zoned {
    timestamp.to_zoned(tz.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_timezone_exists() {
        let tz = local();
        // Should not panic
        let _ = tz.iana_name();
    }

    #[test]
    fn utc_works() {
        let tz = utc();
        assert_eq!(tz.iana_name(), Some("UTC"));
    }

    #[test]
    fn get_known_timezone() {
        let tz = get("America/New_York").unwrap();
        assert_eq!(tz.iana_name(), Some("America/New_York"));
    }

    #[test]
    fn invalid_timezone_fails() {
        assert!(get("Not/A/Timezone").is_err());
    }

    #[test]
    fn now_in_timezone() {
        let tz = get("Europe/London").unwrap();
        let zoned = now_in(&tz);
        assert_eq!(zoned.time_zone().iana_name(), Some("Europe/London"));
    }
}
