//! File write layer — writes structured log events to a file.
//!
//! This layer collects tracing events that have an `event_name` field,
//! filters them by level and event name, and writes them to a file.
//!
//! Features:
//! - Configurable file path
//! - Optional file rotation by size
//! - Filtering by log level and event name
//! - Async writing via tokio::spawn

use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

use super::common::{level_to_u8, EventNameFilter, FieldVisitor};

/// Builder for `FileWriteLayer`.
pub struct FileWriteLayerBuilder {
    path: PathBuf,
    min_level: tracing::Level,
    event_filter: Option<EventNameFilter>,
    allowed_event_names: Option<HashSet<String>>,
    max_size_bytes: Option<u64>,
}

impl FileWriteLayerBuilder {
    fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            min_level: tracing::Level::INFO,
            event_filter: None,
            allowed_event_names: None,
            max_size_bytes: None,
        }
    }

    /// Set minimum log level (default: INFO).
    pub fn min_level(mut self, level: tracing::Level) -> Self {
        self.min_level = level;
        self
    }

    /// Set a predicate filter for event names.
    pub fn event_filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.event_filter = Some(Arc::new(filter));
        self
    }

    /// Only allow specific event names (exact match).
    pub fn allow_event_names(mut self, names: HashSet<String>) -> Self {
        self.allowed_event_names = Some(names);
        self
    }

    /// Set max file size before rotation (default: no rotation).
    pub fn max_size_bytes(mut self, bytes: u64) -> Self {
        self.max_size_bytes = Some(bytes);
        self
    }

    /// Build the layer.
    pub fn build(self) -> FileWriteLayer {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .expect("Failed to open log file");

        FileWriteLayer {
            path: self.path,
            file: Arc::new(Mutex::new(file)),
            min_level: self.min_level,
            event_filter: self.event_filter,
            allowed_event_names: self.allowed_event_names,
            max_size_bytes: self.max_size_bytes,
        }
    }
}

/// A tracing layer that writes structured events to a file.
#[derive(Clone)]
pub struct FileWriteLayer {
    path: PathBuf,
    file: Arc<Mutex<File>>,
    min_level: tracing::Level,
    event_filter: Option<EventNameFilter>,
    allowed_event_names: Option<HashSet<String>>,
    max_size_bytes: Option<u64>,
}

impl FileWriteLayer {
    /// Create a builder for the layer.
    pub fn builder(path: impl Into<PathBuf>) -> FileWriteLayerBuilder {
        FileWriteLayerBuilder::new(path)
    }

    /// Quick constructor with defaults.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self::builder(path).build()
    }

    /// Check if an event passes all filters.
    fn should_collect(&self, event_name: &str, level: &tracing::Level) -> bool {
        let level_num = level_to_u8(level);
        let min_num = level_to_u8(&self.min_level);
        if level_num < min_num {
            return false;
        }
        if let Some(ref filter) = self.event_filter {
            if !filter(event_name) {
                return false;
            }
        }
        if let Some(ref allowed) = self.allowed_event_names {
            if !allowed.contains(event_name) {
                return false;
            }
        }
        true
    }

    /// Rotate the log file if it exceeds max size.
    fn maybe_rotate(&self) {
        let Some(max_size) = self.max_size_bytes else { return };

        let metadata = match std::fs::metadata(&self.path) {
            Ok(m) => m,
            Err(_) => return,
        };

        if metadata.len() < max_size {
            return;
        }

        let base = self.path.clone();
        let rotated = base.with_extension("log.1");
        let _ = std::fs::rename(&base, &rotated);

        let new_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&base)
            .expect("Failed to reopen log file after rotation");

        if let Ok(mut file) = self.file.lock() {
            *file = new_file;
        }
    }
}

impl<S> Layer<S> for FileWriteLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);

        let event_name = match visitor.fields.get("event_name") {
            Some(name) => name.clone(),
            None => return,
        };

        let level = event.metadata().level();

        if !self.should_collect(&event_name, level) {
            return;
        }

        let timestamp = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%z");
        let mut fields = visitor.fields.clone();
        fields.insert("event_name".to_string(), event_name);
        fields.insert("level".to_string(), level.to_string());
        if let Some(msg) = visitor.message {
            fields.insert("message".to_string(), msg);
        }

        let line = match serde_json::to_string(&fields) {
            Ok(json) => format!("{} {}\n", timestamp, json),
            Err(_) => return,
        };

        let file = self.file.clone();
        let layer = self.clone();

        tokio::spawn(async move {
            layer.maybe_rotate();
            if let Ok(mut f) = file.lock() {
                let _ = f.write_all(line.as_bytes());
                let _ = f.flush();
            }
        });
    }
}
