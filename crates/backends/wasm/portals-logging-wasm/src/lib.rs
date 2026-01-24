//! WASM implementation of portals-logging.
//!
//! Uses browser console API for output.

use portals_logging::{Level, Logger, Record};

/// Logger that outputs to the browser console.
#[derive(Debug, Clone)]
pub struct ConsoleLogger {
    min_level: Level,
}

impl Default for ConsoleLogger {
    fn default() -> Self {
        Self::new(Level::Info)
    }
}

impl ConsoleLogger {
    /// Create a new console logger with the specified minimum level.
    pub fn new(min_level: Level) -> Self {
        Self { min_level }
    }
}

impl Logger for ConsoleLogger {
    fn log(&self, record: &Record) {
        if !self.enabled(record.level) {
            return;
        }

        let mut msg = format!("[{}] {}: {}", level_str(record.level), record.target, record.message);

        if !record.fields.is_empty() {
            msg.push_str(" {");
            for (i, (key, value)) in record.fields.iter().enumerate() {
                if i > 0 {
                    msg.push_str(", ");
                }
                msg.push_str(&format!("{}: {}", key, value));
            }
            msg.push('}');
        }

        match record.level {
            Level::Trace | Level::Debug => web_sys::console::debug_1(&msg.into()),
            Level::Info => web_sys::console::info_1(&msg.into()),
            Level::Warn => web_sys::console::warn_1(&msg.into()),
            Level::Error => web_sys::console::error_1(&msg.into()),
        }
    }

    fn enabled(&self, level: Level) -> bool {
        level >= self.min_level
    }
}

fn level_str(level: Level) -> &'static str {
    match level {
        Level::Trace => "TRACE",
        Level::Debug => "DEBUG",
        Level::Info => "INFO",
        Level::Warn => "WARN",
        Level::Error => "ERROR",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn logger_respects_level() {
        let logger = ConsoleLogger::new(Level::Warn);
        assert!(!logger.enabled(Level::Debug));
        assert!(!logger.enabled(Level::Info));
        assert!(logger.enabled(Level::Warn));
        assert!(logger.enabled(Level::Error));
    }

    #[wasm_bindgen_test]
    fn logger_can_log() {
        // Just verify it doesn't panic
        let logger = ConsoleLogger::default();
        logger.info("test", "hello from wasm");
        logger.warn("test", "warning message");
        logger.error("test", "error message");
    }
}
