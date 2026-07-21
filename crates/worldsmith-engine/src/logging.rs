//! Lightweight structured logging primitives.

use serde::{Deserialize, Serialize};
pub use worldsmith_state::LogLevel;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogRecord {
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub fields: Vec<(String, String)>,
}

impl LogRecord {
    pub fn new(level: LogLevel, target: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level,
            target: target.into(),
            message: message.into(),
            fields: Vec::new(),
        }
    }

    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push((key.into(), value.into()));
        self
    }
}

#[derive(Debug, Clone)]
pub struct Logger {
    min_level: LogLevel,
    records: Vec<LogRecord>,
}

impl Logger {
    pub fn new(min_level: LogLevel) -> Self {
        Self {
            min_level,
            records: Vec::new(),
        }
    }

    #[inline]
    pub fn records(&self) -> &[LogRecord] {
        &self.records
    }

    pub fn log(&mut self, record: LogRecord) {
        if record.level >= self.min_level {
            self.records.push(record);
        }
    }

    pub fn debug(&mut self, target: impl Into<String>, message: impl Into<String>) {
        self.log(LogRecord::new(LogLevel::Debug, target, message));
    }

    pub fn info(&mut self, target: impl Into<String>, message: impl Into<String>) {
        self.log(LogRecord::new(LogLevel::Info, target, message));
    }

    pub fn warning(&mut self, target: impl Into<String>, message: impl Into<String>) {
        self.log(LogRecord::new(LogLevel::Warning, target, message));
    }

    pub fn error(&mut self, target: impl Into<String>, message: impl Into<String>) {
        self.log(LogRecord::new(LogLevel::Error, target, message));
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new(LogLevel::Info)
    }
}
