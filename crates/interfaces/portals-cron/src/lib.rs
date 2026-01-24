//! Cron expression interfaces.
//!
//! Parse and evaluate cron expressions for scheduling.

use std::fmt;

/// A parsed cron expression.
///
/// Standard cron format: `minute hour day-of-month month day-of-week`
/// Extended format adds seconds: `second minute hour day-of-month month day-of-week`
pub trait CronExpr: fmt::Display {
    /// Check if this expression matches the given datetime.
    ///
    /// The datetime components are:
    /// - `second`: 0-59
    /// - `minute`: 0-59
    /// - `hour`: 0-23
    /// - `day`: 1-31
    /// - `month`: 1-12
    /// - `weekday`: 0-6 (Sunday = 0)
    fn matches(&self, second: u8, minute: u8, hour: u8, day: u8, month: u8, weekday: u8) -> bool;

    /// Get the original expression string.
    fn as_str(&self) -> &str;
}

/// Error parsing a cron expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CronError {
    /// Invalid field count.
    InvalidFieldCount { expected: &'static str, got: usize },
    /// Invalid field value.
    InvalidField {
        field: &'static str,
        value: String,
        reason: String,
    },
    /// Value out of range.
    OutOfRange {
        field: &'static str,
        value: u32,
        min: u32,
        max: u32,
    },
    /// Invalid step value.
    InvalidStep { field: &'static str, step: u32 },
    /// Other error.
    Other(String),
}

impl fmt::Display for CronError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFieldCount { expected, got } => {
                write!(f, "invalid field count: expected {}, got {}", expected, got)
            }
            Self::InvalidField {
                field,
                value,
                reason,
            } => {
                write!(f, "invalid {} field '{}': {}", field, value, reason)
            }
            Self::OutOfRange {
                field,
                value,
                min,
                max,
            } => {
                write!(
                    f,
                    "{} value {} out of range ({}-{})",
                    field, value, min, max
                )
            }
            Self::InvalidStep { field, step } => {
                write!(f, "invalid step {} for {} field", step, field)
            }
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CronError {}

/// Parser for cron expressions.
pub trait CronParser {
    /// The parsed expression type.
    type Expr: CronExpr;

    /// Parse a standard 5-field cron expression.
    ///
    /// Format: `minute hour day-of-month month day-of-week`
    fn parse(&self, expr: &str) -> Result<Self::Expr, CronError>;

    /// Parse an extended 6-field cron expression with seconds.
    ///
    /// Format: `second minute hour day-of-month month day-of-week`
    fn parse_with_seconds(&self, expr: &str) -> Result<Self::Expr, CronError>;
}

/// Iterator over upcoming cron occurrences.
pub trait CronSchedule: CronExpr {
    /// Find the next occurrence after the given datetime.
    ///
    /// Returns `(year, month, day, hour, minute, second)` or `None` if no
    /// occurrence exists within a reasonable search window.
    fn next_after(
        &self,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Option<(i32, u8, u8, u8, u8, u8)>;
}
