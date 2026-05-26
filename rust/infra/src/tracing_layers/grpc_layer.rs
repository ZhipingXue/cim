//! gRPC collection layer — sends structured log events to a remote gRPC service.
//!
//! This layer collects tracing events that have an `event_name` field,
//! filters them by level and event name, and sends them to a gRPC endpoint.
//!
//! The actual gRPC client is abstracted behind the `GrpcLogSender` trait
//! so users can plug in their own proto-generated client.

use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

use crate::event_collector::{CollectedEvent, EventSeverity};
use super::common::{level_to_u8, EventNameFilter, FieldVisitor};

/// Trait for sending log events via gRPC.
///
/// Implement this with your proto-generated gRPC client.
#[async_trait::async_trait]
pub trait GrpcLogSender: Send + Sync + 'static {
    /// Send a single log event. Returns Ok(()) on success.
    async fn send_log(
        &self,
        event: CollectedEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// A no-op gRPC sender for testing.
#[derive(Debug, Clone)]
pub struct NoOpGrpcSender;

#[async_trait::async_trait]
impl GrpcLogSender for NoOpGrpcSender {
    async fn send_log(
        &self,
        _event: CollectedEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

/// Builder for `GrpcCollectionLayer`.
pub struct GrpcCollectionLayerBuilder {
    sender: Arc<dyn GrpcLogSender>,
    min_level: tracing::Level,
    event_filter: Option<EventNameFilter>,
    allowed_event_names: Option<HashSet<String>>,
    service_name: String,
}

impl GrpcCollectionLayerBuilder {
    fn new(sender: Arc<dyn GrpcLogSender>) -> Self {
        Self {
            sender,
            min_level: tracing::Level::INFO,
            event_filter: None,
            allowed_event_names: None,
            service_name: "unknown".to_string(),
        }
    }

    /// Set minimum log level (default: INFO).
    pub fn min_level(mut self, level: tracing::Level) -> Self {
        self.min_level = level;
        self
    }

    /// Set a predicate filter for event names.
    /// Only events matching the predicate are sent.
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

    /// Set the service name included in sent events.
    pub fn service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = name.into();
        self
    }

    /// Build the layer.
    pub fn build(self) -> GrpcCollectionLayer {
        GrpcCollectionLayer {
            sender: self.sender,
            min_level: self.min_level,
            event_filter: self.event_filter,
            allowed_event_names: self.allowed_event_names,
            service_name: self.service_name,
        }
    }
}

/// A tracing layer that sends structured events to a gRPC service.
#[derive(Clone)]
pub struct GrpcCollectionLayer {
    sender: Arc<dyn GrpcLogSender>,
    min_level: tracing::Level,
    event_filter: Option<EventNameFilter>,
    allowed_event_names: Option<HashSet<String>>,
    service_name: String,
}

impl GrpcCollectionLayer {
    /// Create a builder for the layer.
    pub fn builder(sender: Arc<dyn GrpcLogSender>) -> GrpcCollectionLayerBuilder {
        GrpcCollectionLayerBuilder::new(sender)
    }

    /// Quick constructor with defaults.
    pub fn new(sender: Arc<dyn GrpcLogSender>) -> Self {
        Self::builder(sender).build()
    }

    /// Check if an event passes all filters.
    fn should_collect(
        &self,
        event_name: &str,
        level: &tracing::Level,
    ) -> bool {
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
}

impl fmt::Debug for GrpcCollectionLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GrpcCollectionLayer")
            .field("min_level", &self.min_level)
            .field("service_name", &self.service_name)
            .finish()
    }
}

impl<S> Layer<S> for GrpcCollectionLayer
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

        let severity = match *level {
            tracing::Level::TRACE => EventSeverity::Debug,
            tracing::Level::DEBUG => EventSeverity::Debug,
            tracing::Level::INFO => EventSeverity::Info,
            tracing::Level::WARN => EventSeverity::Warning,
            tracing::Level::ERROR => EventSeverity::Error,
        };

        let mut collected = CollectedEvent::new(
            event_name.clone(),
            severity,
            visitor
                .fields
                .get("sm_id")
                .or_else(|| visitor.fields.get("pjob_id"))
                .or_else(|| visitor.fields.get("cjob_id"))
                .or_else(|| visitor.fields.get("carrier_id"))
                .or_else(|| visitor.fields.get("substrate_id"))
                .cloned()
                .unwrap_or_default(),
            self.service_name.clone(),
            visitor.message.unwrap_or_else(|| event_name),
        );

        if let Some(from) = visitor.fields.get("from_state") {
            collected = collected.with_from_state(from.clone());
        }
        if let Some(to) = visitor.fields.get("to_state") {
            collected = collected.with_to_state(to.clone());
        }
        if let Some(trigger) = visitor.fields.get("trigger") {
            collected = collected.with_trigger(trigger.clone());
        }
        if let Some(err) = visitor.fields.get("error") {
            collected = collected.with_error(err.clone());
        }

        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(e) = sender.send_log(collected).await {
                eprintln!("Failed to send log via gRPC: {}", e);
            }
        });
    }
}
