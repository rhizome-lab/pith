//! Native logging implementation using tracing.

use rhizome_pith_logging::{Level, Logger, Record};

/// Initialize the default tracing subscriber.
pub fn init() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}

/// Initialize with a specific log level.
pub fn init_with_level(level: Level) {
    let filter = match level {
        Level::Trace => "trace",
        Level::Debug => "debug",
        Level::Info => "info",
        Level::Warn => "warn",
        Level::Error => "error",
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init();
}

/// A logger that uses tracing.
#[derive(Debug, Default)]
pub struct TracingLogger {
    min_level: Level,
}

impl TracingLogger {
    /// Create a new tracing logger.
    pub fn new() -> Self {
        Self {
            min_level: Level::Trace,
        }
    }

    /// Create a logger with a minimum level.
    pub fn with_level(level: Level) -> Self {
        Self { min_level: level }
    }
}

impl Logger for TracingLogger {
    fn log(&self, record: &Record) {
        let fields_str = record
            .fields
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(" ");

        let message = if fields_str.is_empty() {
            format!("[{}] {}", record.target, record.message)
        } else {
            format!("[{}] {} {}", record.target, record.message, fields_str)
        };

        match record.level {
            Level::Trace => tracing::trace!("{}", message),
            Level::Debug => tracing::debug!("{}", message),
            Level::Info => tracing::info!("{}", message),
            Level::Warn => tracing::warn!("{}", message),
            Level::Error => tracing::error!("{}", message),
        }
    }

    fn enabled(&self, level: Level) -> bool {
        level >= self.min_level
    }
}

/// A simple stderr logger (no tracing dependency needed at runtime).
#[derive(Debug, Default)]
pub struct StderrLogger {
    min_level: Level,
}

impl StderrLogger {
    /// Create a new stderr logger.
    pub fn new() -> Self {
        Self {
            min_level: Level::Info,
        }
    }

    /// Create with a minimum level.
    pub fn with_level(level: Level) -> Self {
        Self { min_level: level }
    }
}

impl Logger for StderrLogger {
    fn log(&self, record: &Record) {
        let level_str = match record.level {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        };
        eprintln!("[{}] {}: {}", level_str, record.target, record.message);
    }

    fn enabled(&self, level: Level) -> bool {
        level >= self.min_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stderr_logger_works() {
        let logger = StderrLogger::new();
        logger.info("test", "hello world");
        // Should not panic
    }

    #[test]
    fn level_filtering() {
        let logger = StderrLogger::with_level(Level::Warn);
        assert!(!logger.enabled(Level::Info));
        assert!(logger.enabled(Level::Warn));
        assert!(logger.enabled(Level::Error));
    }

    #[test]
    fn record_with_fields() {
        let record = Record::new(Level::Info, "test", "message")
            .field("key", "value")
            .field("count", "42");
        assert_eq!(record.fields.len(), 2);
    }
}
