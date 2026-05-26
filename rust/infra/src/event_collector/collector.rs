//! Event collector trait and implementations.

use async_trait::async_trait;
use std::fmt::Debug;

use super::types::{CollectedEvent, EventSeverity};

/// Trait for collecting events from the state machine.
///
/// Implement this to forward events to E116, E134, or external systems.
#[async_trait]
pub trait EventCollector: Send + Sync + 'static {
    /// Collect an event. Implementors should handle buffering, batching, or forwarding.
    async fn collect(
        &self,
        event: CollectedEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// A no-op event collector that discards all events.
/// Useful for testing or when event collection is not needed.
#[derive(Debug, Clone)]
pub struct NoOpEventCollector;

#[async_trait]
impl EventCollector for NoOpEventCollector {
    async fn collect(
        &self,
        _event: CollectedEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

/// An in-memory event collector for testing.
#[derive(Debug, Clone)]
pub struct InMemoryEventCollector {
    events: std::sync::Arc<tokio::sync::Mutex<Vec<CollectedEvent>>>,
}

impl InMemoryEventCollector {
    pub fn new() -> Self {
        Self {
            events: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    /// Get all collected events.
    pub async fn events(&self) -> Vec<CollectedEvent> {
        self.events.lock().await.clone()
    }

    /// Get events filtered by name.
    pub async fn events_by_name(&self, name: &str) -> Vec<CollectedEvent> {
        self.events
            .lock()
            .await
            .iter()
            .filter(|e| e.event_name == name)
            .cloned()
            .collect()
    }

    /// Get events filtered by severity.
    pub async fn events_by_severity(&self, severity: EventSeverity) -> Vec<CollectedEvent> {
        self.events
            .lock()
            .await
            .iter()
            .filter(|e| e.severity == severity)
            .cloned()
            .collect()
    }

    /// Clear all events.
    pub async fn clear(&self) {
        self.events.lock().await.clear();
    }
}

#[async_trait]
impl EventCollector for InMemoryEventCollector {
    async fn collect(
        &self,
        event: CollectedEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.events.lock().await.push(event);
        Ok(())
    }
}

impl Default for InMemoryEventCollector {
    fn default() -> Self {
        Self::new()
    }
}
