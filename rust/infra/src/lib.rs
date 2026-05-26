//! Shared infrastructure for CIM services.
//!
//! This crate provides:
//! - Event collection traits and implementations
//! - Custom tracing layers (gRPC, file write)
//! - Structured logging helpers and event name constants

pub mod event_collector;
pub mod logging;
pub mod tracing_layers;

pub use event_collector::*;
pub use logging::*;
pub use tracing_layers::*;
