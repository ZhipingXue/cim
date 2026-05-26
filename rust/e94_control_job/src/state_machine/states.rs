use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use tracing::warn;

use crate::domain::model::{CJobCommand, CJobState};
use crate::domain::model::CJobCommand::*;
use crate::domain::model::CJobState::*;
use crate::state_machine::executor::{CJobExecutor, CJobExecutorAdapter, SetupResult};

use semi_common::async_state_machine::{State, StateChangeTrigger};

// ─────────────────────────────────────────────
// E94 State structs — generic over executor type
// ─────────────────────────────────────────────

macro_rules! define_state {
    ($name:ident, $state_id:expr) => {
        #[derive(Clone)]
        pub struct $name<E: CJobExecutor>(PhantomData<E>);

        impl<E: CJobExecutor> Default for $name<E> {
            fn default() -> Self {
                Self(PhantomData)
            }
        }

        impl<E: CJobExecutor> std::fmt::Debug for $name<E> {
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
    E: CJobExecutor,
{
    type Command = CJobCommand;
    type Executor = CJobExecutorAdapter<E>;
    type Error = E::Error;

    async fn on_event(
        self: Arc<Self>,
        trigger: StateChangeTrigger<CJobCommand>,
        executor: &CJobExecutorAdapter<E>,
        sm_id: &str,
    ) -> Result<
        Arc<dyn State<Command = CJobCommand, Executor = CJobExecutorAdapter<E>, Error = E::Error>>,
        E::Error,
    > {
        match trigger {
            StateChangeTrigger::Command(Start) => {
                match executor.executor.setup(sm_id).await? {
                    SetupResult::Ready => Ok(Arc::new(SelectedState::<E>::default())),
                    SetupResult::WaitForCarrier => Ok(Arc::new(WaitingForStartState::<E>::default())),
                }
            }
            StateChangeTrigger::Command(Cancel)
            | StateChangeTrigger::Command(Stop)
            | StateChangeTrigger::Command(Abort) => {
                executor.executor.cancel(sm_id).await?;
                Ok(Arc::new(CancelledState::<E>::default()))
            }
            _ => {
                warn!(state = "Queued", trigger = ?trigger, "Invalid trigger");
                Ok(self)
            }
        }
    }

    fn state_id(&self) -> CJobState {
        Queued
    }
}

#[async_trait]
impl<E> State for SelectedState<E>
where
    E: CJobExecutor,
{
    type Command = CJobCommand;
    type Executor = CJobExecutorAdapter<E>;
    type Error = E::Error;

    async fn on_event(
        self: Arc<Self>,
        trigger: StateChangeTrigger<CJobCommand>,
        executor: &CJobExecutorAdapter<E>,
        sm_id: &str,
    ) -> Result<
        Arc<dyn State<Command = CJobCommand, Executor = CJobExecutorAdapter<E>, Error = E::Error>>,
        E::Error,
    > {
        match trigger {
            StateChangeTrigger::Command(Start) => {
                executor.executor.start(sm_id).await?;
                Ok(Arc::new(WaitingForStartState::<E>::default()))
            }
            StateChangeTrigger::Command(Stop) => {
                executor.executor.stop(sm_id).await?;
                Ok(Arc::new(StoppedState::<E>::default()))
            }
            StateChangeTrigger::Command(Abort) => {
                executor.executor.abort(sm_id).await?;
                Ok(Arc::new(AbortedState::<E>::default()))
            }
            _ => {
                warn!(state = "Selected", trigger = ?trigger, "Invalid trigger");
                Ok(self)
            }
        }
    }

    fn state_id(&self) -> CJobState {
        Selected
    }
}

#[async_trait]
impl<E> State for WaitingForStartState<E>
where
    E: CJobExecutor,
{
    type Command = CJobCommand;
    type Executor = CJobExecutorAdapter<E>;
    type Error = E::Error;

    async fn on_event(
        self: Arc<Self>,
        trigger: StateChangeTrigger<CJobCommand>,
        executor: &CJobExecutorAdapter<E>,
        sm_id: &str,
    ) -> Result<
        Arc<dyn State<Command = CJobCommand, Executor = CJobExecutorAdapter<E>, Error = E::Error>>,
        E::Error,
    > {
        match trigger {
            StateChangeTrigger::Command(Start) => {
                executor.executor.start(sm_id).await?;
                Ok(Arc::new(ExecutingState::<E>::default()))
            }
            StateChangeTrigger::Command(Stop) => {
                executor.executor.stop(sm_id).await?;
                Ok(Arc::new(StoppedState::<E>::default()))
            }
            StateChangeTrigger::Command(Abort) => {
                executor.executor.abort(sm_id).await?;
                Ok(Arc::new(AbortedState::<E>::default()))
            }
            _ => {
                warn!(state = "WaitingForStart", trigger = ?trigger, "Invalid trigger");
                Ok(self)
            }
        }
    }

    fn state_id(&self) -> CJobState {
        WaitingForStart
    }
}

#[async_trait]
impl<E> State for ExecutingState<E>
where
    E: CJobExecutor,
{
    type Command = CJobCommand;
    type Executor = CJobExecutorAdapter<E>;
    type Error = E::Error;

    async fn on_event(
        self: Arc<Self>,
        trigger: StateChangeTrigger<CJobCommand>,
        executor: &CJobExecutorAdapter<E>,
        sm_id: &str,
    ) -> Result<
        Arc<dyn State<Command = CJobCommand, Executor = CJobExecutorAdapter<E>, Error = E::Error>>,
        E::Error,
    > {
        match trigger {
            StateChangeTrigger::Command(Pause) => {
                executor.executor.pause(sm_id).await?;
                Ok(Arc::new(PausedState::<E>::default()))
            }
            StateChangeTrigger::Command(Stop) => {
                executor.executor.stop(sm_id).await?;
                Ok(Arc::new(StoppedState::<E>::default()))
            }
            StateChangeTrigger::Command(Abort) => {
                executor.executor.abort(sm_id).await?;
                Ok(Arc::new(AbortedState::<E>::default()))
            }
            StateChangeTrigger::Command(Start) => {
                executor.executor.complete(sm_id).await?;
                Ok(Arc::new(CompletedState::<E>::default()))
            }
            _ => {
                warn!(state = "Executing", trigger = ?trigger, "Invalid trigger");
                Ok(self)
            }
        }
    }

    fn state_id(&self) -> CJobState {
        Executing
    }
}

#[async_trait]
impl<E> State for PausedState<E>
where
    E: CJobExecutor,
{
    type Command = CJobCommand;
    type Executor = CJobExecutorAdapter<E>;
    type Error = E::Error;

    async fn on_event(
        self: Arc<Self>,
        trigger: StateChangeTrigger<CJobCommand>,
        executor: &CJobExecutorAdapter<E>,
        sm_id: &str,
    ) -> Result<
        Arc<dyn State<Command = CJobCommand, Executor = CJobExecutorAdapter<E>, Error = E::Error>>,
        E::Error,
    > {
        match trigger {
            StateChangeTrigger::Command(Resume) => {
                executor.executor.resume(sm_id).await?;
                Ok(Arc::new(ExecutingState::<E>::default()))
            }
            StateChangeTrigger::Command(Stop) => {
                executor.executor.stop(sm_id).await?;
                Ok(Arc::new(StoppedState::<E>::default()))
            }
            StateChangeTrigger::Command(Abort) => {
                executor.executor.abort(sm_id).await?;
                Ok(Arc::new(AbortedState::<E>::default()))
            }
            _ => {
                warn!(state = "Paused", trigger = ?trigger, "Invalid trigger");
                Ok(self)
            }
        }
    }

    fn state_id(&self) -> CJobState {
        Paused
    }
}

// Terminal states — no transitions allowed
macro_rules! terminal_state {
    ($name:ident, $state_id:expr) => {
        #[async_trait]
        impl<E> State for $name<E>
        where
            E: CJobExecutor,
        {
            type Command = CJobCommand;
            type Executor = CJobExecutorAdapter<E>;
            type Error = E::Error;

            async fn on_event(
                self: Arc<Self>,
                _trigger: StateChangeTrigger<CJobCommand>,
                _executor: &CJobExecutorAdapter<E>,
                _sm_id: &str,
            ) -> Result<
                Arc<dyn State<Command = CJobCommand, Executor = CJobExecutorAdapter<E>, Error = E::Error>>,
                E::Error,
            > {
                warn!(state = stringify!($name), "Terminal state — no transitions");
                Ok(self)
            }

            fn state_id(&self) -> CJobState {
                $state_id
            }
        }
    };
}

terminal_state!(CompletedState, Completed);
terminal_state!(AbortedState, Aborted);
terminal_state!(CancelledState, Cancelled);
terminal_state!(StoppedState, Stopped);
