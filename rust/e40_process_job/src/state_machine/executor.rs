use std::sync::Arc;

use async_trait::async_trait;
use tracing::info;

use crate::domain::model::{PJobCommand, PJobState};
use crate::domain::model::PJobCommand::*;
use crate::domain::model::PJobState::*;
use infra::logging as infra_logging;

use semi_common::async_state_machine::{State, StateChangeTrigger, StateExecutor};

/// E40 ProcessJob executor trait.
///
/// Implementors perform the actual work during state transitions:
/// - Hardware control (start process, pause equipment, etc.)
/// - Logging and audit trails
/// - Database persistence
/// - Event publishing (E116)
/// - Notifying other services
#[async_trait]
pub trait PJobExecutor: Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Called when transitioning from QUEUED to SELECTED (setup phase).
    /// Perform resource allocation, recipe validation, etc.
    async fn setup(
        &self,
        pjob_id: &str,
    ) -> Result<SetupResult, Self::Error>;

    /// Called when transitioning to EXECUTING.
    /// Start the actual process.
    async fn start(&self, pjob_id: &str) -> Result<(), Self::Error>;

    /// Called when transitioning to PAUSED.
    /// Pause the running process.
    async fn pause(&self, pjob_id: &str) -> Result<(), Self::Error>;

    /// Called when transitioning from PAUSED back to EXECUTING.
    /// Resume the paused process.
    async fn resume(&self, pjob_id: &str) -> Result<(), Self::Error>;

    /// Called when transitioning to STOPPED.
    /// Stop the process gracefully.
    async fn stop(&self, pjob_id: &str) -> Result<(), Self::Error>;

    /// Called when transitioning to ABORTED.
    /// Abort the process immediately.
    async fn abort(&self, pjob_id: &str) -> Result<(), Self::Error>;

    /// Called when transitioning to COMPLETED.
    /// Finalize the process.
    async fn complete(&self, pjob_id: &str) -> Result<(), Self::Error>;

    /// Called when cancelling from QUEUED.
    async fn cancel(&self, pjob_id: &str) -> Result<(), Self::Error>;
}

/// Result of the setup operation.
#[derive(Debug, Clone)]
pub enum SetupResult {
    /// Ready to proceed (go to SELECTED, then EXECUTING or WAITING_FOR_START)
    Ready,
    /// Waiting for carrier/material (go to WAITING_FOR_START)
    WaitForCarrier,
}

// ─────────────────────────────────────────────
// StateExecutor implementation for PJobExecutor
// ─────────────────────────────────────────────

/// Wraps a `PJobExecutor` to implement the generic `StateExecutor` trait.
pub struct PJobExecutorAdapter<E> {
    pub(crate) executor: Arc<E>,
}

impl<E> PJobExecutorAdapter<E> {
    pub fn new(executor: Arc<E>) -> Self {
        Self { executor }
    }
}

#[async_trait]
impl<E> StateExecutor for PJobExecutorAdapter<E>
where
    E: PJobExecutor,
{
    type Command = PJobCommand;
    type StateId = PJobState;
    type Error = E::Error;

    async fn handle_command(
        &self,
        current_state: &PJobState,
        cmd: PJobCommand,
        _sm_id: &str,
    ) -> Result<PJobState, E::Error> {
        match (current_state, cmd) {
            (Queued, Start) => Ok(Selected),
            (Queued, Cancel) => Ok(Cancelled),
            (Queued, Stop) => Ok(Cancelled),
            (Queued, Abort) => Ok(Cancelled),
            _ => Ok(Queued),
        }
    }

    async fn on_transition_completed(
        &self,
        from: &PJobState,
        to: &PJobState,
        sm_id: &str,
    ) -> Result<(), E::Error> {
        info!(
            event_name = "sm.transition.completed",
            pjob_id = %sm_id,
            from = ?from,
            to = ?to,
            "PJob transition completed"
        );
        Ok(())
    }
}

// ─────────────────────────────────────────────
// No-op executor for testing / default behavior
// ─────────────────────────────────────────────

/// A no-op executor that does nothing but log.
/// Useful for testing or when no hardware control is needed.
#[derive(Debug, Clone)]
pub struct NoOpPJobExecutor;

#[async_trait]
impl PJobExecutor for NoOpPJobExecutor {
    type Error = std::io::Error;

    async fn setup(&self, pjob_id: &str) -> Result<SetupResult, std::io::Error> {
        infra_logging::log_exec_entry(infra_logging::event_names::EXEC_SETUP, pjob_id, "setup");
        Ok(SetupResult::Ready)
    }

    async fn start(&self, pjob_id: &str) -> Result<(), std::io::Error> {
        infra_logging::log_exec_entry(infra_logging::event_names::EXEC_START, pjob_id, "start");
        Ok(())
    }

    async fn pause(&self, pjob_id: &str) -> Result<(), std::io::Error> {
        infra_logging::log_exec_entry(infra_logging::event_names::EXEC_PAUSE, pjob_id, "pause");
        Ok(())
    }

    async fn resume(&self, pjob_id: &str) -> Result<(), std::io::Error> {
        infra_logging::log_exec_entry(infra_logging::event_names::EXEC_RESUME, pjob_id, "resume");
        Ok(())
    }

    async fn stop(&self, pjob_id: &str) -> Result<(), std::io::Error> {
        infra_logging::log_exec_entry(infra_logging::event_names::EXEC_STOP, pjob_id, "stop");
        Ok(())
    }

    async fn abort(&self, pjob_id: &str) -> Result<(), std::io::Error> {
        infra_logging::log_exec_entry(infra_logging::event_names::EXEC_ABORT, pjob_id, "abort");
        Ok(())
    }

    async fn complete(&self, pjob_id: &str) -> Result<(), std::io::Error> {
        infra_logging::log_exec_entry(infra_logging::event_names::EXEC_COMPLETE, pjob_id, "complete");
        Ok(())
    }

    async fn cancel(&self, pjob_id: &str) -> Result<(), std::io::Error> {
        infra_logging::log_exec_entry(infra_logging::event_names::EXEC_CANCEL, pjob_id, "cancel");
        Ok(())
    }
}
