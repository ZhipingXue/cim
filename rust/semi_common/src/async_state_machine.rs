use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::{info, warn};

// ─────────────────────────────────────────────
// Trigger types that cause state changes
// ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum StateChangeTrigger<Cmd> {
    /// User/system command (Start, Stop, Pause, etc.)
    Command(Cmd),
    /// External event (carrier arrived, alarm cleared, etc.)
    Event(String),
    /// Timer or timeout
    Timeout(String),
    /// Internal condition
    Internal(String),
}

// ─────────────────────────────────────────────
// Executor trait — performs actions during transitions
// ─────────────────────────────────────────────

/// An executor that handles state transitions by performing side effects
/// (hardware control, logging, database writes, etc.).
///
/// The executor is called *during* the transition, allowing it to:
/// 1. Validate that the transition is allowed (e.g., hardware ready)
/// 2. Perform the actual work (e.g., start the process)
/// 3. Return the next state based on the result
#[async_trait]
pub trait StateExecutor: Send + Sync + 'static {
    /// The command type this executor handles
    type Command: Debug + Send + Sync + 'static;
    /// The state identifier type
    type StateId: Debug + Clone + Send + Sync + 'static;
    /// Error type returned by executor operations
    type Error: std::error::Error + Send + Sync + 'static;

    /// Handle a command in the context of the current state.
    ///
    /// This is where the real work happens:
    /// - Control hardware
    /// - Write to database
    /// - Log operations
    /// - Notify other systems
    ///
    /// Returns the target state to transition to.
    async fn handle_command(
        &self,
        current_state: &Self::StateId,
        cmd: Self::Command,
        sm_id: &str,
    ) -> Result<Self::StateId, Self::Error>;

    /// Called after a successful transition.
    /// Use for post-transition side effects (event publishing, etc.)
    async fn on_transition_completed(
        &self,
        from: &Self::StateId,
        to: &Self::StateId,
        sm_id: &str,
    ) -> Result<(), Self::Error>;
}

// ─────────────────────────────────────────────
// State trait — each state implements transition logic
// ─────────────────────────────────────────────

/// A state in the async state machine.
///
/// Each state struct implements `on_event` to define:
/// - Which triggers are valid in this state
/// - How to delegate to the executor
/// - What state to transition to based on executor result
#[async_trait]
pub trait State: Send + Sync + Debug + 'static {
    /// The command type this state handles
    type Command: Send + Sync + 'static;
    /// The executor type
    type Executor: StateExecutor<Command = Self::Command>;
    /// The error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Handle a trigger and return the next state (or error).
    ///
    /// The default implementation delegates to the executor for Command triggers
    /// and returns an error for other trigger types.
    async fn on_event(
        self: Arc<Self>,
        trigger: StateChangeTrigger<Self::Command>,
        executor: &Self::Executor,
        sm_id: &str,
    ) -> Result<Arc<dyn State<Command = Self::Command, Executor = Self::Executor, Error = Self::Error>>, Self::Error>;

    /// Return the state's identifier
    fn state_id(&self) -> <Self::Executor as StateExecutor>::StateId;
}



// ─────────────────────────────────────────────
// AsyncStateMachine — orchestrates states and executor
// ─────────────────────────────────────────────

/// An async state machine where each state can delegate to an executor
/// during transitions.
///
/// Unlike the static `StateMachine<S, C>`, this machine:
/// - Uses trait objects for states (each state is a struct implementing `State`)
/// - Calls the executor during transitions for side effects
/// - Allows dynamic transition logic based on executor results
pub struct AsyncStateMachine<E>
where
    E: StateExecutor,
{
    id: String,
    current: Arc<RwLock<Arc<dyn State<Command = E::Command, Executor = E, Error = E::Error>>>>,
    executor: Arc<E>,
}

impl<E> Debug for AsyncStateMachine<E>
where
    E: StateExecutor,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncStateMachine")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl<E> AsyncStateMachine<E>
where
    E: StateExecutor,
{
    pub fn new(
        id: String,
        initial: Arc<dyn State<Command = E::Command, Executor = E, Error = E::Error>>,
        executor: Arc<E>,
    ) -> Self {
        Self {
            id,
            current: Arc::new(RwLock::new(initial)),
            executor,
        }
    }

    /// Get the current state identifier
    pub async fn current_state_id(&self) -> E::StateId {
        let state = self.current.read().await;
        state.state_id()
    }

    /// Handle a trigger and potentially transition to a new state.
    ///
    /// Call flow:
    /// 1. Get current state
    /// 2. Call `state.on_event(trigger, executor)` — executor performs work
    /// 3. If successful, update current state
    /// 4. Call executor's `on_transition_completed`
    /// 5. Return new state ID
    pub async fn handle_trigger(
        &self,
        trigger: StateChangeTrigger<E::Command>,
    ) -> Result<E::StateId, E::Error> {
        let current_arc = self.current.read().await.clone();
        let from_id = current_arc.state_id();

        info!(
            sm_id = %self.id,
            from = ?from_id,
            trigger = ?trigger,
            "Handling state trigger"
        );

        // Delegate to current state — this may call executor.handle_command()
        let new_state = current_arc.on_event(trigger, &self.executor, &self.id).await?;
        let to_id = new_state.state_id();

        // Update current state
        let mut current = self.current.write().await;
        *current = new_state;
        drop(current);

        info!(
            sm_id = %self.id,
            from = ?from_id,
            to = ?to_id,
            "State transition completed"
        );

        // Post-transition hook
        self.executor.on_transition_completed(&from_id, &to_id, &self.id).await?;

        Ok(to_id)
    }

    /// Convenience method for handling commands
    pub async fn handle_command(&self, cmd: E::Command) -> Result<E::StateId, E::Error> {
        self.handle_trigger(StateChangeTrigger::Command(cmd)).await
    }
}

// ─────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestCommand {
        Start,
        Stop,
        Pause,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestStateId {
        Idle,
        Running,
        Paused,
        Stopped,
    }

    #[derive(Debug, thiserror::Error)]
    enum TestError {
        #[error("Invalid transition: {0:?} + {1:?}")]
        InvalidTransition(TestStateId, TestCommand),
    }

    #[derive(Debug)]
    struct TestExecutor {
        call_count: AtomicI32,
    }

    impl TestExecutor {
        fn new() -> Self {
            Self {
                call_count: AtomicI32::new(0),
            }
        }
    }

    #[async_trait]
    impl StateExecutor for TestExecutor {
        type Command = TestCommand;
        type StateId = TestStateId;
        type Error = TestError;

        async fn handle_command(
            &self,
            current_state: &TestStateId,
            cmd: TestCommand,
            _sm_id: &str,
        ) -> Result<TestStateId, TestError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);

            match (current_state, cmd) {
                (TestStateId::Idle, TestCommand::Start) => Ok(TestStateId::Running),
                (TestStateId::Running, TestCommand::Stop) => Ok(TestStateId::Stopped),
                (TestStateId::Running, TestCommand::Pause) => Ok(TestStateId::Paused),
                (TestStateId::Paused, TestCommand::Start) => Ok(TestStateId::Running),
                _ => Err(TestError::InvalidTransition(*current_state, cmd)),
            }
        }

        async fn on_transition_completed(
            &self,
            _from: &TestStateId,
            _to: &TestStateId,
            _sm_id: &str,
        ) -> Result<(), TestError> {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct IdleState;

    #[async_trait]
    impl State for IdleState {
        type Command = TestCommand;
        type Executor = TestExecutor;

        type Error = TestError;

        async fn on_event(
            self: Arc<Self>,
            trigger: StateChangeTrigger<TestCommand>,
            executor: &TestExecutor,
            sm_id: &str,
        ) -> Result<Arc<dyn State<Command = TestCommand, Executor = TestExecutor, Error = TestError>>, TestError> {
            match trigger {
                StateChangeTrigger::Command(cmd) => {
                    let next_id = executor.handle_command(&TestStateId::Idle, cmd, sm_id).await?;
                    match next_id {
                        TestStateId::Running => Ok(Arc::new(RunningState)),
                        _ => Err(TestError::InvalidTransition(TestStateId::Idle, cmd)),
                    }
                }
                _ => Err(TestError::InvalidTransition(TestStateId::Idle, TestCommand::Start)),
            }
        }

        fn state_id(&self) -> TestStateId {
            TestStateId::Idle
        }
    }

    #[derive(Debug)]
    struct RunningState;

    #[async_trait]
    impl State for RunningState {
        type Command = TestCommand;
        type Executor = TestExecutor;
        type Error = TestError;

        async fn on_event(
            self: Arc<Self>,
            trigger: StateChangeTrigger<TestCommand>,
            executor: &TestExecutor,
            sm_id: &str,
        ) -> Result<Arc<dyn State<Command = TestCommand, Executor = TestExecutor, Error = TestError>>, TestError> {
            match trigger {
                StateChangeTrigger::Command(cmd) => {
                    let next_id = executor.handle_command(&TestStateId::Running, cmd, sm_id).await?;
                    match next_id {
                        TestStateId::Stopped => Ok(Arc::new(StoppedState)),
                        TestStateId::Paused => Ok(Arc::new(PausedState)),
                        _ => Err(TestError::InvalidTransition(TestStateId::Running, cmd)),
                    }
                }
                _ => Err(TestError::InvalidTransition(TestStateId::Running, TestCommand::Start)),
            }
        }

        fn state_id(&self) -> TestStateId {
            TestStateId::Running
        }
    }

    #[derive(Debug)]
    struct PausedState;

    #[async_trait]
    impl State for PausedState {
        type Command = TestCommand;
        type Executor = TestExecutor;
        type Error = TestError;

        async fn on_event(
            self: Arc<Self>,
            trigger: StateChangeTrigger<TestCommand>,
            executor: &TestExecutor,
            sm_id: &str,
        ) -> Result<Arc<dyn State<Command = TestCommand, Executor = TestExecutor, Error = TestError>>, TestError> {
            match trigger {
                StateChangeTrigger::Command(cmd) => {
                    let next_id = executor.handle_command(&TestStateId::Paused, cmd, sm_id).await?;
                    match next_id {
                        TestStateId::Running => Ok(Arc::new(RunningState)),
                        _ => Err(TestError::InvalidTransition(TestStateId::Paused, cmd)),
                    }
                }
                _ => Err(TestError::InvalidTransition(TestStateId::Paused, TestCommand::Start)),
            }
        }

        fn state_id(&self) -> TestStateId {
            TestStateId::Paused
        }
    }

    #[derive(Debug)]
    struct StoppedState;

    #[async_trait]
    impl State for StoppedState {
        type Command = TestCommand;
        type Executor = TestExecutor;
        type Error = TestError;

        async fn on_event(
            self: Arc<Self>,
            _trigger: StateChangeTrigger<TestCommand>,
            _executor: &TestExecutor,
            _sm_id: &str,
        ) -> Result<Arc<dyn State<Command = TestCommand, Executor = TestExecutor, Error = TestError>>, TestError> {
            Err(TestError::InvalidTransition(TestStateId::Stopped, TestCommand::Start))
        }

        fn state_id(&self) -> TestStateId {
            TestStateId::Stopped
        }
    }

    #[tokio::test]
    async fn test_async_sm_full_flow() {
        let executor = Arc::new(TestExecutor::new());
        let sm = AsyncStateMachine::new(
            "test-1".to_string(),
            Arc::new(IdleState) as Arc<dyn State<Command = TestCommand, Executor = TestExecutor, Error = TestError>>,
            executor.clone(),
        );

        assert_eq!(sm.current_state_id().await, TestStateId::Idle);

        // Idle -> Running
        let result = sm.handle_command(TestCommand::Start).await;
        assert!(result.is_ok());
        assert_eq!(sm.current_state_id().await, TestStateId::Running);
        assert_eq!(executor.call_count.load(Ordering::SeqCst), 1);

        // Running -> Paused
        let result = sm.handle_command(TestCommand::Pause).await;
        assert!(result.is_ok());
        assert_eq!(sm.current_state_id().await, TestStateId::Paused);
        assert_eq!(executor.call_count.load(Ordering::SeqCst), 2);

        // Paused -> Running
        let result = sm.handle_command(TestCommand::Start).await;
        assert!(result.is_ok());
        assert_eq!(sm.current_state_id().await, TestStateId::Running);
        assert_eq!(executor.call_count.load(Ordering::SeqCst), 3);

        // Running -> Stopped
        let result = sm.handle_command(TestCommand::Stop).await;
        assert!(result.is_ok());
        assert_eq!(sm.current_state_id().await, TestStateId::Stopped);
        assert_eq!(executor.call_count.load(Ordering::SeqCst), 4);

        // Stopped is terminal — should fail
        let result = sm.handle_command(TestCommand::Start).await;
        assert!(result.is_err());
        assert_eq!(sm.current_state_id().await, TestStateId::Stopped);
        // Executor not called for terminal state
        assert_eq!(executor.call_count.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn test_async_sm_invalid_transition() {
        let executor = Arc::new(TestExecutor::new());
        let sm = AsyncStateMachine::new(
            "test-2".to_string(),
            Arc::new(IdleState) as Arc<dyn State<Command = TestCommand, Executor = TestExecutor, Error = TestError>>,
            executor.clone(),
        );

        // Idle + Stop is invalid
        let result = sm.handle_command(TestCommand::Stop).await;
        assert!(result.is_err());
        assert_eq!(sm.current_state_id().await, TestStateId::Idle);
        // Executor was called but returned error
        assert_eq!(executor.call_count.load(Ordering::SeqCst), 1);
    }
}
