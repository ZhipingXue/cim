pub mod pb;
pub mod domain;
pub mod state_machine;
pub mod store;
pub mod service;

#[cfg(test)]
mod tests;

pub use store::PJobStore;
pub use service::PJobService;
pub use state_machine::{PJobExecutor, PJobExecutorAdapter, SetupResult, NoOpPJobExecutor};
pub use domain::{ProcessJob, PJobState, PJobCommand, Recipe, RecipeMethod, RecipeVariable, Material};

// Re-export infra types for convenience
pub use infra::{CollectedEvent, EventCollector, EventSeverity, InMemoryEventCollector, NoOpEventCollector};
pub use infra::{GrpcCollectionLayer, GrpcLogSender, NoOpGrpcSender, FileWriteLayer};
