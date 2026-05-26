use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use tracing::{info, warn};

use crate::domain::model::{PJobCommand, PJobState};
use crate::domain::model::PJobCommand::*;
use crate::domain::model::PJobState::*;
use crate::state_machine::executor::{PJobExecutor, PJobExecutorAdapter, SetupResult};
use infra::logging as infra_logging;

use semi_common::async_state_machine::{State, StateChangeTrigger};

// ─────────────────────────────────────────────
// E40 State structs — generic over executor type
// ─────────────────────────────────────────────

macro_rules! define_state {
    ($name:ident, $state_id:expr) => {
        #[derive(Clone)]
        pub struct $name<E: PJobExecutor>(PhantomData<E>);

        impl<E: PJobExecutor> Default for $name<E> {
            fn default() -> Self {
                Self(PhantomData)
            }
        }

        impl<E: PJobExecutor> std::fmt::Debug for $name<E> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name)).finish()
            }
        }
    };
}

define_state!(QueuedState, Queued);
define_state!(SelectedState, Selected);
define_state!(WaitingForStartState, WaitingForStart);
define_state!(ExecutingState, Executing);
define_state!(PausedState, Paused);
define_state!(CompletedState, Completed);
define_state!(AbortedState, Aborted);
define_state!(CancelledState, Cancelled);
define_state!(StoppedState, Stopped);

// ─────────────────────────────────────────────
// Generic State trait implementations
// ─────────────────────────────────────────────

#[async_trait]
impl<E> State for QueuedState<E>
where
    E: PJobExecutor,
{
    type Command = PJobCommand;
    type Executor = PJobExecutorAdapter<E>;
    type Error = E::Error;

    async fn on_event(
        self: Arc<Self>,
        trigger: StateChangeTrigger<PJobCommand>,
        executor: &PJobExecutorAdapter<E>,
        sm_id: &str,
    ) -> Result<
        Arc<dyn State<Command = PJobCommand, Executor = PJobExecutorAdapter<E>, Error = E::Error>>,
        E::Error,
    > {
        let trigger_str = format!("{:?}", trigger);
        infra_logging::log_transition_attempt(sm_id, "Queued", &trigger_str);

        match trigger {
            StateChangeTrigger::Command(Start) => {
                match executor.executor.setup(sm_id).await {
                    Ok(SetupResult::Ready) => {
                        infra_logging::log_transition_success(sm_id, "Queued", "Selected", "Start");
                        Ok(Arc::new(SelectedState::<E>::default()))
                    }
                    Ok(SetupResult::WaitForCarrier) => {
                        infra_logging::log_transition_success(sm_id, "Queued", "WaitingForStart", "Start");
                        Ok(Arc::new(WaitingForStartState::<E>::default()))
                    }
                    Err(e) => {
                        infra_logging::log_transition_failure(sm_id, "Queued", "Start", &e.to_string());
                        Err(e)
                    }
                }
            }
            StateChangeTrigger::Command(Cancel)
            | StateChangeTrigger::Command(Stop)
            | StateChangeTrigger::Command(Abort) => {
                let cmd_str = format!("{:?}", trigger);
                match executor.executor.cancel(sm_id).await {
                    Ok(()) => {
                        infra_logging::log_transition_success(sm_id, "Queued", "Cancelled", &cmd_str);
                        Ok(Arc::new(CancelledState::<E>::default()))
                    }
                    Err(e) => {
                        infra_logging::log_transition_failure(sm_id, "Queued", &cmd_str, &e.to_string());
                        Err(e)
                    }
                }
            }
            _ => {
                infra_logging::log_invalid_trigger(sm_id, "Queued", &trigger_str);
                Ok(self)
            }
        }
    }

    fn state_id(&self) -> PJobState {
        Queued
    }
}

#[async_trait]
impl<E> State for SelectedState<E>
where
    E: PJobExecutor,
{
    type Command = PJobCommand;
    type Executor = PJobExecutorAdapter<E>;
    type Error = E::Error;

    async fn on_event(
        self: Arc<Self>,
        trigger: StateChangeTrigger<PJobCommand>,
        executor: &PJobExecutorAdapter<E>,
        sm_id: &str,
    ) -> Result<
        Arc<dyn State<Command = PJobCommand, Executor = PJobExecutorAdapter<E>, Error = E::Error>>,
        E::Error,
    > {
        let trigger_str = format!("{:?}", trigger);
        infra_logging::log_transition_attempt(sm_id, "Selected", &trigger_str);

        match trigger {
            StateChangeTrigger::Command(Start) => {
                match executor.executor.start(sm_id).await {
                    Ok(()) => {
                        infra_logging::log_transition_success(sm_id, "Selected", "WaitingForStart", "Start");
                        Ok(Arc::new(WaitingForStartState::<E>::default()))
                    }
                    Err(e) => {
                        infra_logging::log_transition_failure(sm_id, "Selected", "Start", &e.to_string());
                        Err(e)
                    }
                }
            }
            StateChangeTrigger::Command(Stop) => {
                match executor.executor.stop(sm_id).await {
                    Ok(()) => {
                        infra_logging::log_transition_success(sm_id, "Selected", "Stopped", "Stop");
                        Ok(Arc::new(StoppedState::<E>::default()))
                    }
                    Err(e) => {
                        infra_logging::log_transition_failure(sm_id, "Selected", "Stop", &e.to_string());
                        Err(e)
                    }
                }
            }
            StateChangeTrigger::Command(Abort) => {
                match executor.executor.abort(sm_id).await {
                    Ok(()) => {
                        infra_logging::log_transition_success(sm_id, "Selected", "Aborted", "Abort");
                        Ok(Arc::new(AbortedState::<E>::default()))
                    }
                    Err(e) => {
                        infra_logging::log_transition_failure(sm_id, "Selected", "Abort", &e.to_string());
                        Err(e)
                    }
                }
            }
            _ => {
                infra_logging::log_invalid_trigger(sm_id, "Selected", &trigger_str);
                Ok(self)
            }
        }
    }

    fn state_id(&self) -> PJobState {
        Selected
    }
}

#[async_trait]
impl<E> State for WaitingForStartState<E>
where
    E: PJobExecutor,
{
    type Command = PJobCommand;
    type Executor = PJobExecutorAdapter<E>;
    type Error = E::Error;

    async fn on_event(
        self: Arc<Self>,
        trigger: StateChangeTrigger<PJobCommand>,
        executor: &PJobExecutorAdapter<E>,
        sm_id: &str,
    ) -> Result<
        Arc<dyn State<Command = PJobCommand, Executor = PJobExecutorAdapter<E>, Error = E::Error>>,
        E::Error,
    > {
        let trigger_str = format!("{:?}", trigger);
        infra_logging::log_transition_attempt(sm_id, "WaitingForStart", &trigger_str);

        match trigger {
            StateChangeTrigger::Command(Start) => {
                match executor.executor.start(sm_id).await {
                    Ok(()) => {
                        infra_logging::log_transition_success(sm_id, "WaitingForStart", "Executing", "Start");
                        Ok(Arc::new(ExecutingState::<E>::default()))
                    }
                    Err(e) => {
                        infra_logging::log_transition_failure(sm_id, "WaitingForStart", "Start", &e.to_string());
                        Err(e)
                    }
                }
            }
            StateChangeTrigger::Command(Stop) => {
                match executor.executor.stop(sm_id).await {
                    Ok(()) => {
                        infra_logging::log_transition_success(sm_id, "WaitingForStart", "Stopped", "Stop");
                        Ok(Arc::new(StoppedState::<E>::default()))
                    }
                    Err(e) => {
                        infra_logging::log_transition_failure(sm_id, "WaitingForStart", "Stop", &e.to_string());
                        Err(e)
                    }
                }
            }
            StateChangeTrigger::Command(Abort) => {
                match executor.executor.abort(sm_id).await {
                    Ok(()) => {
                        infra_logging::log_transition_success(sm_id, "WaitingForStart", "Aborted", "Abort");
                        Ok(Arc::new(AbortedState::<E>::default()))
                    }
                    Err(e) => {
                        infra_logging::log_transition_failure(sm_id, "WaitingForStart", "Abort", &e.to_string());
                        Err(e)
                    }
                }
            }
            _ => {
                infra_logging::log_invalid_trigger(sm_id, "WaitingForStart", &trigger_str);
                Ok(self)
            }
        }
    }

    fn state_id(&self) -> PJobState {
        WaitingForStart
    }
}

#[async_trait]
impl<E> State for ExecutingState<E>
where
    E: PJobExecutor,
{
    type Command = PJobCommand;
    type Executor = PJobExecutorAdapter<E>;
    type Error = E::Error;

    async fn on_event(
        self: Arc<Self>,
        trigger: StateChangeTrigger<PJobCommand>,
        executor: &PJobExecutorAdapter<E>,
        sm_id: &str,
    ) -> Result<
        Arc<dyn State<Command = PJobCommand, Executor = PJobExecutorAdapter<E>, Error = E::Error>>,
        E::Error,
    > {
        let trigger_str = format!("{:?}", trigger);
        infra_logging::log_transition_attempt(sm_id, "Executing", &trigger_str);

        match trigger {
            StateChangeTrigger::Command(Pause) => {
                match executor.executor.pause(sm_id).await {
                    Ok(()) => {
                        infra_logging::log_transition_success(sm_id, "Executing", "Paused", "Pause");
                        Ok(Arc::new(PausedState::<E>::default()))
                    }
                    Err(e) => {
                        infra_logging::log_transition_failure(sm_id, "Executing", "Pause", &e.to_string());
                        Err(e)
                    }
                }
            }
            StateChangeTrigger::Command(Stop) => {
                match executor.executor.stop(sm_id).await {
                    Ok(()) => {
                        infra_logging::log_transition_success(sm_id, "Executing", "Stopped", "Stop");
                        Ok(Arc::new(StoppedState::<E>::default()))
                    }
                    Err(e) => {
                        infra_logging::log_transition_failure(sm_id, "Executing", "Stop", &e.to_string());
                        Err(e)
                    }
                }
            }
            StateChangeTrigger::Command(Abort) => {
                match executor.executor.abort(sm_id).await {
                    Ok(()) => {
                        infra_logging::log_transition_success(sm_id, "Executing", "Aborted", "Abort");
                        Ok(Arc::new(AbortedState::<E>::default()))
                    }
                    Err(e) => {
                        infra_logging::log_transition_failure(sm_id, "Executing", "Abort", &e.to_string());
                        Err(e)
                    }
                }
            }
            StateChangeTrigger::Command(Start) => {
                match executor.executor.complete(sm_id).await {
                    Ok(()) => {
                        infra_logging::log_transition_success(sm_id, "Executing", "Completed", "Start");
                        Ok(Arc::new(CompletedState::<E>::default()))
                    }
                    Err(e) => {
                        infra_logging::log_transition_failure(sm_id, "Executing", "Start", &e.to_string());
                        Err(e)
                    }
                }
            }
            _ => {
                infra_logging::log_invalid_trigger(sm_id, "Executing", &trigger_str);
                Ok(self)
            }
        }
    }

    fn state_id(&self) -> PJobState {
        Executing
    }
}

#[async_trait]
impl<E> State for PausedState<E>
where
    E: PJobExecutor,
{
    type Command = PJobCommand;
    type Executor = PJobExecutorAdapter<E>;
    type Error = E::Error;

    async fn on_event(
        self: Arc<Self>,
        trigger: StateChangeTrigger<PJobCommand>,
        executor: &PJobExecutorAdapter<E>,
        sm_id: &str,
    ) -> Result<
        Arc<dyn State<Command = PJobCommand, Executor = PJobExecutorAdapter<E>, Error = E::Error>>,
        E::Error,
    > {
        let trigger_str = format!("{:?}", trigger);
        infra_logging::log_transition_attempt(sm_id, "Paused", &trigger_str);

        match trigger {
            StateChangeTrigger::Command(Resume) => {
                match executor.executor.resume(sm_id).await {
                    Ok(()) => {
                        infra_logging::log_transition_success(sm_id, "Paused", "Executing", "Resume");
                        Ok(Arc::new(ExecutingState::<E>::default()))
                    }
                    Err(e) => {
                        infra_logging::log_transition_failure(sm_id, "Paused", "Resume", &e.to_string());
                        Err(e)
                    }
                }
            }
            StateChangeTrigger::Command(Stop) => {
                match executor.executor.stop(sm_id).await {
                    Ok(()) => {
                        infra_logging::log_transition_success(sm_id, "Paused", "Stopped", "Stop");
                        Ok(Arc::new(StoppedState::<E>::default()))
                    }
                    Err(e) => {
                        infra_logging::log_transition_failure(sm_id, "Paused", "Stop", &e.to_string());
                        Err(e)
                    }
                }
            }
            StateChangeTrigger::Command(Abort) => {
                match executor.executor.abort(sm_id).await {
                    Ok(()) => {
                        infra_logging::log_transition_success(sm_id, "Paused", "Aborted", "Abort");
                        Ok(Arc::new(AbortedState::<E>::default()))
                    }
                    Err(e) => {
                        infra_logging::log_transition_failure(sm_id, "Paused", "Abort", &e.to_string());
                        Err(e)
                    }
                }
            }
            _ => {
                infra_logging::log_invalid_trigger(sm_id, "Paused", &trigger_str);
                Ok(self)
            }
        }
    }

    fn state_id(&self) -> PJobState {
        Paused
    }
}

// Terminal states — no transitions allowed
macro_rules! terminal_state {
    ($name:ident, $state_id:expr) => {
        #[async_trait]
        impl<E> State for $name<E>
        where
            E: PJobExecutor,
        {
            type Command = PJobCommand;
            type Executor = PJobExecutorAdapter<E>;
            type Error = E::Error;

            async fn on_event(
                self: Arc<Self>,
                trigger: StateChangeTrigger<PJobCommand>,
                _executor: &PJobExecutorAdapter<E>,
                sm_id: &str,
            ) -> Result<
                Arc<dyn State<Command = PJobCommand, Executor = PJobExecutorAdapter<E>, Error = E::Error>>,
                E::Error,
            > {
                let trigger_str = format!("{:?}", trigger);
                infra_logging::log_terminal_reject(sm_id, stringify!($name), &trigger_str);
                Ok(self)
            }

            fn state_id(&self) -> PJobState {
                $state_id
            }
        }
    };
}

terminal_state!(CompletedState, Completed);
terminal_state!(AbortedState, Aborted);
terminal_state!(CancelledState, Cancelled);
terminal_state!(StoppedState, Stopped);
