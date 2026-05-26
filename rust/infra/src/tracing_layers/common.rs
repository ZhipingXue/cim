//! Shared types for tracing layers.

use std::collections::HashMap;
use std::sync::Arc;

/// Filter predicate for event names.
pub type EventNameFilter = Arc<dyn Fn(&str) -> bool + Send + Sync>;

/// Visitor that extracts fields from tracing events.
pub struct FieldVisitor {
    pub fields: HashMap<String, String>,
    pub message: Option<String>,
}

impl FieldVisitor {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            message: None,
        }
    }
}

impl tracing::field::Visit for FieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let key = field.name();
        let val = format!("{:?}", value);
        // Strip quotes from string values
        let val = val.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(&val);
        if key == "message" {
            self.message = Some(val.to_string());
        } else {
            self.fields.insert(key.to_string(), val.to_string());
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        let key = field.name();
        if key == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields.insert(key.to_string(), value.to_string());
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }
}

/// Convert tracing Level to numeric value for comparison.
pub fn level_to_u8(level: &tracing::Level) -> u8 {
    match *level {
        tracing::Level::TRACE => 0,
        tracing::Level::DEBUG => 1,
        tracing::Level::INFO => 2,
        tracing::Level::WARN => 3,
        tracing::Level::ERROR => 4,
    }
}
