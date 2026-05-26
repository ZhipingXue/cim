use std::sync::Arc;

use async_trait::async_trait;
use tracing::info;

use crate::domain::model::{CJobCommand, CJobState};
use crate::domain::model::CJobCommand::*;
use crate::domain::model::CJobState::*;

use semi_common::async_state_machine::{State, StateChangeTrigger, StateExecutor};

/// E94 ControlJob executor trait.
#[async_trait]
pub trait CJobExecutor: Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn setup(&self,
        cjob_id: &str,
    ) -> Result<SetupResult, Self::Error>;

    async fn start(&self, cjob_id: &str) -> Result<(), Self::Error>;
    async fn pause(&self, cjob_id: &str) -> Result<(), Self::Error>;
    async fn resume(&self, cjob_id: &str) -> Result<(), Self::Error>;
    async fn stop(&self, cjob_id: &str) -> Result<(), Self::Error>;
    async fn abort(&self, cjob_id: &str) -> Result<(), Self::Error>;
    async fn complete(&self, cjob_id: &str) -> Result<(), Self::Error>;
    async fn cancel(&self, cjob_id: &str) -> Result<(), Self::Error>;
}

/// Result of the setup operation.
#[derive(Debug, Clone)]
pub enum SetupResult {
    Ready,
    WaitForCarrier,
}

// ─────────────────────────────────────────────
// StateExecutor adapter
// ─────────────────────────────────────────────

pub struct CJobExecutorAdapter<E> {
    pub(crate) executor: Arc<E>,
}

impl<E> CJobExecutorAdapter<E> {
    pub fn new(executor: Arc<E>) -> Self {
        Self { executor }
    }
}

#[async_trait]
impl<E> StateExecutor for CJobExecutorAdapter<E>
where
    E: CJobExecutor,
{
    type Command = CJobCommand;
    type StateId = CJobState;
    type Error = E::Error;

    async fn handle_command(
        &self,
        current_state: &CJobState,
        cmd: CJobCommand,
        _sm_id: &str,
    ) -> Result<CJobState, E::Error> {
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
        from: &CJobState,
        to: &CJobState,
        sm_id: &str,
    ) -> Result<(), E::Error> {
        info!(cjob_id = %sm_id, from = ?from, to = ?to, "CJob transition completed");
        Ok(())
    }
}

// ─────────────────────────────────────────────
// No-op executor
// ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NoOpCJobExecutor;

#[async_trait]
impl CJobExecutor for NoOpCJobExecutor {
    type Error = std::io::Error;

    async fn setup(&self, cjob_id: &str) -> Result<SetupResult, std::io::Error> {
        info!(cjob_id, "NoOp: setup");
        Ok(SetupResult::Ready)
    }

    async fn start(&self, cjob_id: &str) -> Result<(), std::io::Error> {
        info!(cjob_id, "NoOp: start");
        Ok(())
    }

    async fn pause(&self, cjob_id: &str) -> Result<(), std::io::Error> {
        info!(cjob_id, "NoOp: pause");
        Ok(())
    }

    async fn resume(&self, cjob_id: &str) -> Result<(), std::io::Error> {
        info!(cjob_id, "NoOp: resume");
        Ok(())
    }

    async fn stop(&self, cjob_id: &str) -> Result<(), std::io::Error> {
        info!(cjob_id, "NoOp: stop");
        Ok(())
    }

    async fn abort(&self, cjob_id: &str) -> Result<(), std::io::Error> {
        info!(cjob_id, "NoOp: abort");
        Ok(())
    }

    async fn complete(&self, cjob_id: &str) -> Result<(), std::io::Error> {
        info!(cjob_id, "NoOp: complete");
        Ok(())
    }

    async fn cancel(&self, cjob_id: &str) -> Result<(), std::io::Error> {
        info!(cjob_id, "NoOp: cancel");
        Ok(())
    }
}
