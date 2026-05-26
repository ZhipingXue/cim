use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::RwLock;
use tracing::info;

use crate::domain::model::{PJobCommand, PJobState, ProcessJob};
use crate::domain::model::PJobState::*;
use crate::state_machine::executor::{PJobExecutor, PJobExecutorAdapter};
use infra::logging as infra_logging;

use semi_common::async_state_machine::AsyncStateMachine;
use semi_common::error::{CimError, CimResult};

#[derive(Debug)]
pub(crate) struct PJobEntry<E>
where
    E: PJobExecutor,
{
    sm: AsyncStateMachine<PJobExecutorAdapter<E>>,
    pub job: ProcessJob,
}

impl<E> PJobEntry<E>
where
    E: PJobExecutor,
{
    pub fn new(job: ProcessJob, executor: Arc<E>) -> Self {
        use crate::state_machine::states::*;
        let initial: Arc<dyn semi_common::async_state_machine::State<Command = PJobCommand, Executor = PJobExecutorAdapter<E>, Error = E::Error>> = match job.state {
            Queued => Arc::new(QueuedState::default()),
            Selected => Arc::new(SelectedState::default()),
            WaitingForStart => Arc::new(WaitingForStartState::default()),
            Executing => Arc::new(ExecutingState::default()),
            Paused => Arc::new(PausedState::default()),
            Completed => Arc::new(CompletedState::default()),
            Aborted => Arc::new(AbortedState::default()),
            Cancelled => Arc::new(CancelledState::default()),
            Stopped => Arc::new(StoppedState::default()),
        };
        let executor = Arc::new(PJobExecutorAdapter::new(executor));
        let sm = AsyncStateMachine::new(
            job.pjob_id.clone(),
            initial,
            executor,
        );
        Self { sm, job }
    }

    /// Execute a command via the async state machine.
    pub async fn execute_command(
        &mut self,
        cmd: PJobCommand,
    ) -> CimResult<PJobState> {
        let from_state = self.job.state;
        let result = self.sm.handle_command(cmd).await
            .map_err(|e| CimError::Internal(e.to_string()))?;
        
        // Update job state
        self.job.state = result;
        
        // Update timestamps
        match result {
            Executing if self.job.started_at.is_none() => {
                self.job.started_at = Some(SystemTime::now());
            }
            Completed | Aborted | Cancelled | Stopped => {
                self.job.completed_at = Some(SystemTime::now());
            }
            _ => {}
        }
        
        Ok(result)
    }
}

/// A ProcessJob store that is generic over the executor type.
///
/// Users can provide their own `PJobExecutor` implementation to perform
/// actual hardware control, logging, database writes, etc.
#[derive(Debug, Clone)]
pub struct PJobStore<E>
where
    E: PJobExecutor,
{
    entries: Arc<RwLock<HashMap<String, PJobEntry<E>>>>,
    executor: Arc<E>,
}

impl<E> Default for PJobStore<E>
where
    E: PJobExecutor + Default,
{
    fn default() -> Self {
        Self::new(Arc::new(E::default()))
    }
}

impl<E> PJobStore<E>
where
    E: PJobExecutor,
{
    /// Create a new store with the given executor.
    pub fn new(executor: Arc<E>) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            executor,
        }
    }

    pub async fn create(&self, job: ProcessJob) -> CimResult<ProcessJob> {
        let mut entries = self.entries.write().await;
        if entries.contains_key(&job.pjob_id) {
            infra_logging::log_store_create_duplicate(&job.pjob_id, "pjob");
            return Err(CimError::AlreadyExists(format!(
                "PJob {} already exists",
                job.pjob_id
            )));
        }
        let entry = PJobEntry::new(job.clone(), self.executor.clone());
        entries.insert(job.pjob_id.clone(), entry);
        infra_logging::log_store_create(&job.pjob_id, "pjob", &format!("{:?}", job.state));
        Ok(job)
    }

    pub async fn get(&self, pjob_id: &str) -> Option<ProcessJob> {
        let entries = self.entries.read().await;
        let job = entries.get(pjob_id).map(|e| e.job.clone());
        infra_logging::log_store_get(pjob_id, "pjob", job.is_some());
        job
    }

    pub async fn list(&self, filter_state: Option<PJobState>, cjob_id: &str) -> Vec<ProcessJob> {
        let entries = self.entries.read().await;
        let jobs: Vec<ProcessJob> = entries
            .values()
            .filter(|e| {
                filter_state.map_or(true, |s| e.job.state == s)
                    && (cjob_id.is_empty() || e.job.cjob_id == cjob_id)
            })
            .map(|e| e.job.clone())
            .collect();
        
        info!(
            event_name = infra_logging::event_names::STORE_LIST,
            count = jobs.len(),
            filter_state = ?filter_state,
            cjob_id = %cjob_id,
            "ProcessJob list retrieved"
        );
        jobs
    }

    pub async fn execute_command(
        &self,
        pjob_id: &str,
        command: PJobCommand,
    ) -> CimResult<ProcessJob> {
        let mut entries = self.entries.write().await;
        let entry = entries
            .get_mut(pjob_id)
            .ok_or_else(|| {
                infra_logging::log_store_command_not_found(pjob_id, "pjob", &format!("{:?}", command));
                CimError::NotFound(format!("PJob {} not found", pjob_id))
            })?;

        let from_state = entry.job.state;
        let valid = Self::is_valid_transition(from_state, command);
        if !valid {
            infra_logging::log_store_command_invalid(pjob_id, "pjob", &format!("{:?}", command), &format!("{:?}", from_state));
            return Err(CimError::InvalidArgument(format!(
                "Command {:?} is not valid from state {:?}",
                command, from_state
            )));
        }

        let new_state = entry.execute_command(command).await?;
        infra_logging::log_store_command(pjob_id, "pjob", &format!("{:?}", command), &format!("{:?}", from_state), &format!("{:?}", new_state));
        Ok(entry.job.clone())
    }

    /// Check if a command is valid from a given state (E40 transition rules).
    fn is_valid_transition(state: PJobState, cmd: PJobCommand) -> bool {
        use crate::domain::model::PJobCommand::*;
        match (state, cmd) {
            (Queued, Start) => true,
            (Queued, Cancel) => true,
            (Queued, Stop) => true,
            (Queued, Abort) => true,
            (Selected, Start) => true,
            (Selected, Stop) => true,
            (Selected, Abort) => true,
            (WaitingForStart, Start) => true,
            (WaitingForStart, Stop) => true,
            (WaitingForStart, Abort) => true,
            (Executing, Start) => true,
            (Executing, Pause) => true,
            (Executing, Stop) => true,
            (Executing, Abort) => true,
            (Paused, Resume) => true,
            (Paused, Stop) => true,
            (Paused, Abort) => true,
            _ => false,
        }
    }
}
