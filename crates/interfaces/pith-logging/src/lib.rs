//! Structured logging interfaces.
//!
//! Based on WASI logging.

/// Log levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Level {
    /// Trace level - very verbose debugging.
    Trace,
    /// Debug level - debugging information.
    Debug,
    /// Info level - general information.
    #[default]
    Info,
    /// Warn level - warnings.
    Warn,
    /// Error level - errors.
    Error,
}

/// A structured log record.
#[derive(Debug, Clone)]
pub struct Record {
    pub level: Level,
    pub target: String,
    pub message: String,
    pub fields: Vec<(String, String)>,
}

impl Record {
    /// Create a new log record.
    pub fn new(level: Level, target: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level,
            target: target.into(),
            message: message.into(),
            fields: Vec::new(),
        }
    }

    /// Add a field to the record.
    pub fn field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push((key.into(), value.into()));
        self
    }
}

/// A logger that can emit log records.
pub trait Logger {
    /// Log a record.
    fn log(&self, record: &Record);

    /// Check if a level is enabled.
    fn enabled(&self, level: Level) -> bool;

    /// Log a trace message.
    fn trace(&self, target: &str, message: &str) {
        if self.enabled(Level::Trace) {
            self.log(&Record::new(Level::Trace, target, message));
        }
    }

    /// Log a debug message.
    fn debug(&self, target: &str, message: &str) {
        if self.enabled(Level::Debug) {
            self.log(&Record::new(Level::Debug, target, message));
        }
    }

    /// Log an info message.
    fn info(&self, target: &str, message: &str) {
        if self.enabled(Level::Info) {
            self.log(&Record::new(Level::Info, target, message));
        }
    }

    /// Log a warning message.
    fn warn(&self, target: &str, message: &str) {
        if self.enabled(Level::Warn) {
            self.log(&Record::new(Level::Warn, target, message));
        }
    }

    /// Log an error message.
    fn error(&self, target: &str, message: &str) {
        if self.enabled(Level::Error) {
            self.log(&Record::new(Level::Error, target, message));
        }
    }
}
