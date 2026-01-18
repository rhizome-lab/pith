//! Portable cron expression implementation.
//!
//! Works on both native and WASM targets.

use rhizome_pith_cron::{CronError, CronExpr, CronParser, CronSchedule};
use std::fmt;

/// A parsed cron expression.
#[derive(Debug, Clone)]
pub struct Cron {
    expr: String,
    seconds: FieldMatcher,
    minutes: FieldMatcher,
    hours: FieldMatcher,
    days: FieldMatcher,
    months: FieldMatcher,
    weekdays: FieldMatcher,
}

/// Matches values for a cron field.
#[derive(Debug, Clone)]
enum FieldMatcher {
    /// Match any value.
    Any,
    /// Match specific values.
    Values(Vec<u8>),
}

impl FieldMatcher {
    fn matches(&self, value: u8) -> bool {
        match self {
            Self::Any => true,
            Self::Values(values) => values.contains(&value),
        }
    }

    fn parse(s: &str, field: &'static str, min: u8, max: u8) -> Result<Self, CronError> {
        let s = s.trim();

        if s == "*" {
            return Ok(Self::Any);
        }

        let mut values = Vec::new();

        for part in s.split(',') {
            let part = part.trim();

            if let Some((range, step)) = part.split_once('/') {
                // Step value: */2 or 1-10/2
                let step: u8 = step.parse().map_err(|_| CronError::InvalidField {
                    field,
                    value: part.to_string(),
                    reason: "invalid step".to_string(),
                })?;

                if step == 0 {
                    return Err(CronError::InvalidStep {
                        field,
                        step: step as u32,
                    });
                }

                let (start, end) = if range == "*" {
                    (min, max)
                } else if let Some((a, b)) = range.split_once('-') {
                    let a: u8 = a.parse().map_err(|_| CronError::InvalidField {
                        field,
                        value: part.to_string(),
                        reason: "invalid range start".to_string(),
                    })?;
                    let b: u8 = b.parse().map_err(|_| CronError::InvalidField {
                        field,
                        value: part.to_string(),
                        reason: "invalid range end".to_string(),
                    })?;
                    (a, b)
                } else {
                    let v: u8 = range.parse().map_err(|_| CronError::InvalidField {
                        field,
                        value: part.to_string(),
                        reason: "invalid value".to_string(),
                    })?;
                    (v, max)
                };

                for v in (start..=end).step_by(step as usize) {
                    if v >= min && v <= max && !values.contains(&v) {
                        values.push(v);
                    }
                }
            } else if let Some((start, end)) = part.split_once('-') {
                // Range: 1-5
                let start: u8 = start.parse().map_err(|_| CronError::InvalidField {
                    field,
                    value: part.to_string(),
                    reason: "invalid range start".to_string(),
                })?;
                let end: u8 = end.parse().map_err(|_| CronError::InvalidField {
                    field,
                    value: part.to_string(),
                    reason: "invalid range end".to_string(),
                })?;

                if start > end {
                    return Err(CronError::InvalidField {
                        field,
                        value: part.to_string(),
                        reason: "range start > end".to_string(),
                    });
                }

                for v in start..=end {
                    if v < min || v > max {
                        return Err(CronError::OutOfRange {
                            field,
                            value: v as u32,
                            min: min as u32,
                            max: max as u32,
                        });
                    }
                    if !values.contains(&v) {
                        values.push(v);
                    }
                }
            } else {
                // Single value
                let v: u8 = part.parse().map_err(|_| CronError::InvalidField {
                    field,
                    value: part.to_string(),
                    reason: "invalid value".to_string(),
                })?;

                if v < min || v > max {
                    return Err(CronError::OutOfRange {
                        field,
                        value: v as u32,
                        min: min as u32,
                        max: max as u32,
                    });
                }

                if !values.contains(&v) {
                    values.push(v);
                }
            }
        }

        values.sort();
        Ok(Self::Values(values))
    }
}

impl Cron {
    fn parse_5_field(expr: &str) -> Result<Self, CronError> {
        let fields: Vec<&str> = expr.split_whitespace().collect();
        if fields.len() != 5 {
            return Err(CronError::InvalidFieldCount {
                expected: "5",
                got: fields.len(),
            });
        }

        Ok(Self {
            expr: expr.to_string(),
            seconds: FieldMatcher::Values(vec![0]), // Default to 0 seconds
            minutes: FieldMatcher::parse(fields[0], "minute", 0, 59)?,
            hours: FieldMatcher::parse(fields[1], "hour", 0, 23)?,
            days: FieldMatcher::parse(fields[2], "day", 1, 31)?,
            months: FieldMatcher::parse(fields[3], "month", 1, 12)?,
            weekdays: FieldMatcher::parse(fields[4], "weekday", 0, 6)?,
        })
    }

    fn parse_6_field(expr: &str) -> Result<Self, CronError> {
        let fields: Vec<&str> = expr.split_whitespace().collect();
        if fields.len() != 6 {
            return Err(CronError::InvalidFieldCount {
                expected: "6",
                got: fields.len(),
            });
        }

        Ok(Self {
            expr: expr.to_string(),
            seconds: FieldMatcher::parse(fields[0], "second", 0, 59)?,
            minutes: FieldMatcher::parse(fields[1], "minute", 0, 59)?,
            hours: FieldMatcher::parse(fields[2], "hour", 0, 23)?,
            days: FieldMatcher::parse(fields[3], "day", 1, 31)?,
            months: FieldMatcher::parse(fields[4], "month", 1, 12)?,
            weekdays: FieldMatcher::parse(fields[5], "weekday", 0, 6)?,
        })
    }
}

impl CronExpr for Cron {
    fn matches(&self, second: u8, minute: u8, hour: u8, day: u8, month: u8, weekday: u8) -> bool {
        self.seconds.matches(second)
            && self.minutes.matches(minute)
            && self.hours.matches(hour)
            && self.days.matches(day)
            && self.months.matches(month)
            && self.weekdays.matches(weekday)
    }

    fn as_str(&self) -> &str {
        &self.expr
    }
}

impl fmt::Display for Cron {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.expr)
    }
}

impl CronSchedule for Cron {
    fn next_after(
        &self,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Option<(i32, u8, u8, u8, u8, u8)> {
        // Simple brute-force search with reasonable limit
        let mut y = year;
        let mut mo = month;
        let mut d = day;
        let mut h = hour;
        let mut mi = minute;
        let mut s = second + 1;

        // Search up to 4 years ahead
        let max_year = year + 4;

        while y <= max_year {
            // Normalize overflow
            if s > 59 {
                s = 0;
                mi += 1;
            }
            if mi > 59 {
                mi = 0;
                h += 1;
            }
            if h > 23 {
                h = 0;
                d += 1;
            }

            let days_in_month = days_in_month(y, mo);
            if d > days_in_month {
                d = 1;
                mo += 1;
            }
            if mo > 12 {
                mo = 1;
                y += 1;
            }

            if y > max_year {
                return None;
            }

            let weekday = day_of_week(y, mo, d);

            if self.matches(s, mi, h, d, mo, weekday) {
                return Some((y, mo, d, h, mi, s));
            }

            // Increment by one second
            s += 1;
        }

        None
    }
}

/// Calculate day of week (0 = Sunday).
fn day_of_week(year: i32, month: u8, day: u8) -> u8 {
    // Zeller's congruence for Gregorian calendar
    let m = month as i32;
    let d = day as i32;
    let (y, m) = if m < 3 { (year - 1, m + 12) } else { (year, m) };

    let k = y % 100;
    let j = y / 100;

    let h = (d + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;
    // Convert: h=0 is Saturday, h=1 is Sunday, etc.
    // We want: 0=Sunday, 1=Monday, etc.
    ((h + 6) % 7) as u8
}

/// Get days in month.
fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 31,
    }
}

/// Check if year is a leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Default cron parser.
#[derive(Debug, Default, Clone, Copy)]
pub struct CronParserImpl;

impl CronParserImpl {
    pub fn new() -> Self {
        Self
    }
}

impl CronParser for CronParserImpl {
    type Expr = Cron;

    fn parse(&self, expr: &str) -> Result<Self::Expr, CronError> {
        Cron::parse_5_field(expr)
    }

    fn parse_with_seconds(&self, expr: &str) -> Result<Self::Expr, CronError> {
        Cron::parse_6_field(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_every_minute() {
        let parser = CronParserImpl::new();
        let cron = parser.parse("* * * * *").unwrap();
        assert!(cron.matches(0, 0, 0, 1, 1, 0));
        assert!(cron.matches(0, 30, 12, 15, 6, 3));
    }

    #[test]
    fn parse_specific_time() {
        let parser = CronParserImpl::new();
        let cron = parser.parse("30 8 * * *").unwrap();
        assert!(cron.matches(0, 30, 8, 1, 1, 0));
        assert!(!cron.matches(0, 31, 8, 1, 1, 0));
        assert!(!cron.matches(0, 30, 9, 1, 1, 0));
    }

    #[test]
    fn parse_range() {
        let parser = CronParserImpl::new();
        let cron = parser.parse("0-30 * * * *").unwrap();
        assert!(cron.matches(0, 0, 0, 1, 1, 0));
        assert!(cron.matches(0, 15, 0, 1, 1, 0));
        assert!(cron.matches(0, 30, 0, 1, 1, 0));
        assert!(!cron.matches(0, 31, 0, 1, 1, 0));
    }

    #[test]
    fn parse_step() {
        let parser = CronParserImpl::new();
        let cron = parser.parse("*/15 * * * *").unwrap();
        assert!(cron.matches(0, 0, 0, 1, 1, 0));
        assert!(cron.matches(0, 15, 0, 1, 1, 0));
        assert!(cron.matches(0, 30, 0, 1, 1, 0));
        assert!(cron.matches(0, 45, 0, 1, 1, 0));
        assert!(!cron.matches(0, 10, 0, 1, 1, 0));
    }

    #[test]
    fn parse_list() {
        let parser = CronParserImpl::new();
        let cron = parser.parse("0,15,30 * * * *").unwrap();
        assert!(cron.matches(0, 0, 0, 1, 1, 0));
        assert!(cron.matches(0, 15, 0, 1, 1, 0));
        assert!(cron.matches(0, 30, 0, 1, 1, 0));
        assert!(!cron.matches(0, 45, 0, 1, 1, 0));
    }

    #[test]
    fn parse_with_seconds() {
        let parser = CronParserImpl::new();
        let cron = parser.parse_with_seconds("30 0 * * * *").unwrap();
        assert!(cron.matches(30, 0, 0, 1, 1, 0));
        assert!(!cron.matches(0, 0, 0, 1, 1, 0));
    }

    #[test]
    fn weekday_matching() {
        let parser = CronParserImpl::new();
        // Monday only (weekday = 1)
        let cron = parser.parse("0 0 * * 1").unwrap();
        assert!(cron.matches(0, 0, 0, 1, 1, 1));
        assert!(!cron.matches(0, 0, 0, 1, 1, 0)); // Sunday
        assert!(!cron.matches(0, 0, 0, 1, 1, 2)); // Tuesday
    }

    #[test]
    fn invalid_field_count() {
        let parser = CronParserImpl::new();
        let result = parser.parse("* * *");
        assert!(matches!(
            result,
            Err(CronError::InvalidFieldCount {
                expected: "5",
                got: 3
            })
        ));
    }

    #[test]
    fn out_of_range() {
        let parser = CronParserImpl::new();
        let result = parser.parse("60 * * * *");
        assert!(matches!(result, Err(CronError::OutOfRange { .. })));
    }

    #[test]
    fn display() {
        let parser = CronParserImpl::new();
        let cron = parser.parse("*/15 8-17 * * 1-5").unwrap();
        assert_eq!(cron.as_str(), "*/15 8-17 * * 1-5");
        assert_eq!(format!("{}", cron), "*/15 8-17 * * 1-5");
    }

    #[test]
    fn day_of_week_calculation() {
        // Known dates
        assert_eq!(day_of_week(2024, 1, 1), 1); // Monday
        assert_eq!(day_of_week(2024, 12, 25), 3); // Wednesday
        assert_eq!(day_of_week(2000, 1, 1), 6); // Saturday
    }

    #[test]
    fn leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2023));
    }

    #[test]
    fn next_occurrence() {
        let parser = CronParserImpl::new();
        let cron = parser.parse("0 12 * * *").unwrap(); // Every day at 12:00

        // Next after 2024-01-01 08:00:00 should be 2024-01-01 12:00:00
        let next = cron.next_after(2024, 1, 1, 8, 0, 0);
        assert_eq!(next, Some((2024, 1, 1, 12, 0, 0)));

        // Next after 2024-01-01 12:00:00 should be 2024-01-02 12:00:00
        let next = cron.next_after(2024, 1, 1, 12, 0, 0);
        assert_eq!(next, Some((2024, 1, 2, 12, 0, 0)));
    }
}
