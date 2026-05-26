//! Event collection types.
//!
//! Core data structures for collecting and forwarding events.

use std::fmt::Debug;

/// Severity level for collected events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// A structured event that can be collected and forwarded.
#[derive(Debug, Clone)]
pub struct CollectedEvent {
    /// Event name / classification (e.g., "sm.transition.success")
    pub event_name: String,
    /// Severity level
    pub severity: EventSeverity,
    /// Source state machine ID (e.g., pjob_id)
    pub sm_id: String,
    /// Source state machine type (e.g., "e40_process_job")
    pub sm_type: String,
    /// Current / from state
    pub from_state: Option<String>,
    /// Target / to state
    pub to_state: Option<String>,
    /// Trigger that caused the event (command, external event, etc.)
    pub trigger: Option<String>,
    /// Human-readable message
    pub message: String,
    /// Optional error details
    pub error: Option<String>,
    /// Timestamp (set by collector)
    pub timestamp: std::time::SystemTime,
}

impl CollectedEvent {
    /// Create a new event with current timestamp.
    pub fn new(
        event_name: impl Into<String>,
        severity: EventSeverity,
        sm_id: impl Into<String>,
        sm_type: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            event_name: event_name.into(),
            severity,
            sm_id: sm_id.into(),
            sm_type: sm_type.into(),
            from_state: None,
            to_state: None,
            trigger: None,
            message: message.into(),
            error: None,
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Set from_state.
    pub fn with_from_state(mut self, state: impl Into<String>) -> Self {
        self.from_state = Some(state.into());
        self
    }

    /// Set to_state.
    pub fn with_to_state(mut self, state: impl Into<String>) -> Self {
        self.to_state = Some(state.into());
        self
    }

    /// Set trigger.
    pub fn with_trigger(mut self, trigger: impl Into<String>) -> Self {
        self.trigger = Some(trigger.into());
        self
    }

    /// Set error.
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }
}
