use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::RwLock;
use tracing::info;

use crate::domain::model::{CJobCommand, CJobState, ControlJob};
use crate::domain::model::CJobState::*;
use crate::state_machine::executor::{CJobExecutor, CJobExecutorAdapter};

use infra::logging as infra_logging;

use semi_common::async_state_machine::AsyncStateMachine;
use semi_common::error::{CimError, CimResult};

#[derive(Debug)]
pub(crate) struct CJobEntry<E>
where
    E: CJobExecutor,
{
    sm: AsyncStateMachine<CJobExecutorAdapter<E>>,
    pub job: ControlJob,
}

impl<E> CJobEntry<E>
where
    E: CJobExecutor,
{
    pub fn new(job: ControlJob, executor: Arc<E>) -> Self {
        use crate::state_machine::states::*;
        let initial: Arc<dyn semi_common::async_state_machine::State<Command = CJobCommand, Executor = CJobExecutorAdapter<E>, Error = E::Error>> = match job.state {
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
        let executor = Arc::new(CJobExecutorAdapter::new(executor));
        let sm = AsyncStateMachine::new(
            job.cjob_id.clone(),
            initial,
            executor,
        );
        Self { sm, job }
    }

    pub async fn execute_command(
        &mut self,
        cmd: CJobCommand,
    ) -> CimResult<CJobState> {
        let result = self.sm.handle_command(cmd).await
            .map_err(|e| CimError::Internal(e.to_string()))?;
        
        self.job.state = result;
        
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

/// A ControlJob store that is generic over the executor type.
///
/// Users can provide their own `CJobExecutor` implementation to perform
/// actual hardware control, logging, database writes, etc.
#[derive(Debug, Clone)]
pub struct CJobStore<E>
where
    E: CJobExecutor,
{
    entries: Arc<RwLock<HashMap<String, CJobEntry<E>>>>,
    executor: Arc<E>,
}

impl<E> Default for CJobStore<E>
where
    E: CJobExecutor + Default,
{
    fn default() -> Self {
        Self::new(Arc::new(E::default()))
    }
}

impl<E> CJobStore<E>
where
    E: CJobExecutor,
{
    /// Create a new store with the given executor.
    pub fn new(executor: Arc<E>) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            executor,
        }
    }

    pub async fn create(&self, job: ControlJob) -> CimResult<ControlJob> {
        let mut entries = self.entries.write().await;
        if entries.contains_key(&job.cjob_id) {
            infra_logging::log_store_create_duplicate(&job.cjob_id, "cjob");
            return Err(CimError::AlreadyExists(format!(
                "CJob {} already exists",
                job.cjob_id
            )));
        }
        let entry = CJobEntry::new(job.clone(), self.executor.clone());
        entries.insert(job.cjob_id.clone(), entry);
        infra_logging::log_store_create(&job.cjob_id, "cjob", &format!("{:?}", job.state));
        Ok(job)
    }

    pub async fn get(&self, cjob_id: &str) -> Option<ControlJob> {
        let entries = self.entries.read().await;
        let job = entries.get(cjob_id).map(|e| e.job.clone());
        infra_logging::log_store_get(cjob_id, "cjob", job.is_some());
        job
    }

    pub async fn list(&self, filter_state: Option<CJobState>) -> Vec<ControlJob> {
        let entries = self.entries.read().await;
        let jobs: Vec<ControlJob> = entries
            .values()
            .filter(|e| filter_state.map_or(true, |s| e.job.state == s))
            .map(|e| e.job.clone())
            .collect();
        
        info!(
            event_name = infra_logging::event_names::STORE_LIST,
            count = jobs.len(),
            filter_state = ?filter_state,
            "ControlJob list retrieved"
        );
        jobs
    }

    pub async fn execute_command(
        &self,
        cjob_id: &str,
        command: CJobCommand,
    ) -> CimResult<ControlJob> {
        let mut entries = self.entries.write().await;
        let entry = entries
            .get_mut(cjob_id)
            .ok_or_else(|| {
                infra_logging::log_store_command_not_found(cjob_id, "cjob", &format!("{:?}", command));
                CimError::NotFound(format!("CJob {} not found", cjob_id))
            })?;

        let from_state = entry.job.state;
        let valid = Self::is_valid_transition(from_state, command);
        if !valid {
            infra_logging::log_store_command_invalid(cjob_id, "cjob", &format!("{:?}", command), &format!("{:?}", from_state));
            return Err(CimError::InvalidArgument(format!(
                "Command {:?} is not valid from state {:?}",
                command, from_state
            )));
        }

        let new_state = entry.execute_command(command).await?;
        infra_logging::log_store_command(cjob_id, "cjob", &format!("{:?}", command), &format!("{:?}", from_state), &format!("{:?}", new_state));
        Ok(entry.job.clone())
    }

    fn is_valid_transition(state: CJobState, cmd: CJobCommand) -> bool {
        use CJobCommand::*;
        match (state, cmd) {
            (CJobState::Queued, Start | Cancel | Stop | Abort) => true,
            (CJobState::Selected, Start | Stop | Abort) => true,
            (CJobState::WaitingForStart, Start | Stop | Abort) => true,
            (CJobState::Executing, Start | Pause | Stop | Abort) => true,
            (CJobState::Paused, Resume | Stop | Abort) => true,
            _ => false,
        }
    }
}
