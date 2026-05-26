use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;

use crate::domain::model::{PJobCommand, PJobState};
use crate::state_machine::executor::{NoOpPJobExecutor, PJobExecutor, SetupResult};
use crate::state_machine::states::*;

use semi_common::async_state_machine::{State, StateChangeTrigger};

// ─────────────────────────────────────────────
// Custom executor for testing
// ─────────────────────────────────────────────

#[derive(Debug, Clone)]
struct CountingPJobExecutor {
    setup_count: Arc<AtomicUsize>,
    start_count: Arc<AtomicUsize>,
    pause_count: Arc<AtomicUsize>,
    resume_count: Arc<AtomicUsize>,
    stop_count: Arc<AtomicUsize>,
    abort_count: Arc<AtomicUsize>,
    complete_count: Arc<AtomicUsize>,
    cancel_count: Arc<AtomicUsize>,
}

impl CountingPJobExecutor {
    fn new() -> Self {
        Self {
            setup_count: Arc::new(AtomicUsize::new(0)),
            start_count: Arc::new(AtomicUsize::new(0)),
            pause_count: Arc::new(AtomicUsize::new(0)),
            resume_count: Arc::new(AtomicUsize::new(0)),
            stop_count: Arc::new(AtomicUsize::new(0)),
            abort_count: Arc::new(AtomicUsize::new(0)),
            complete_count: Arc::new(AtomicUsize::new(0)),
            cancel_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl PJobExecutor for CountingPJobExecutor {
    type Error = std::io::Error;

    async fn setup(&self, _pjob_id: &str) -> Result<SetupResult, Self::Error> {
        self.setup_count.fetch_add(1, Ordering::SeqCst);
        Ok(SetupResult::Ready)
    }

    async fn start(&self, _pjob_id: &str) -> Result<(), Self::Error> {
        self.start_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn pause(&self, _pjob_id: &str) -> Result<(), Self::Error> {
        self.pause_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn resume(&self, _pjob_id: &str) -> Result<(), Self::Error> {
        self.resume_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn stop(&self, _pjob_id: &str) -> Result<(), Self::Error> {
        self.stop_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn abort(&self, _pjob_id: &str) -> Result<(), Self::Error> {
        self.abort_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn complete(&self, _pjob_id: &str) -> Result<(), Self::Error> {
        self.complete_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn cancel(&self, _pjob_id: &str) -> Result<(), Self::Error> {
        self.cancel_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

// ─────────────────────────────────────────────
// State tests
// ─────────────────────────────────────────────

#[tokio::test]
async fn test_queued_state_start_goes_to_selected() {
    let state = Arc::new(QueuedState::<NoOpPJobExecutor>::default());
    let executor = crate::state_machine::executor::PJobExecutorAdapter::new(Arc::new(NoOpPJobExecutor));
    let next = state
        .on_event(
            StateChangeTrigger::Command(PJobCommand::Start),
            &executor,
            "pj-001",
        )
        .await
        .unwrap();
    assert_eq!(next.state_id(), PJobState::Selected);
}

#[tokio::test]
async fn test_queued_state_cancel_goes_to_cancelled() {
    let state = Arc::new(QueuedState::<NoOpPJobExecutor>::default());
    let executor = crate::state_machine::executor::PJobExecutorAdapter::new(Arc::new(NoOpPJobExecutor));
    let next = state
        .on_event(
            StateChangeTrigger::Command(PJobCommand::Cancel),
            &executor,
            "pj-001",
        )
        .await
        .unwrap();
    assert_eq!(next.state_id(), PJobState::Cancelled);
}

#[tokio::test]
async fn test_queued_state_invalid_trigger_stays_queued() {
    let state = Arc::new(QueuedState::<NoOpPJobExecutor>::default());
    let executor = crate::state_machine::executor::PJobExecutorAdapter::new(Arc::new(NoOpPJobExecutor));
    let next = state
        .on_event(
            StateChangeTrigger::Command(PJobCommand::Pause),
            &executor,
            "pj-001",
        )
        .await
        .unwrap();
    assert_eq!(next.state_id(), PJobState::Queued);
}

#[tokio::test]
async fn test_executing_state_pause_goes_to_paused() {
    let state = Arc::new(ExecutingState::<NoOpPJobExecutor>::default());
    let executor = crate::state_machine::executor::PJobExecutorAdapter::new(Arc::new(NoOpPJobExecutor));
    let next = state
        .on_event(
            StateChangeTrigger::Command(PJobCommand::Pause),
            &executor,
            "pj-001",
        )
        .await
        .unwrap();
    assert_eq!(next.state_id(), PJobState::Paused);
}

#[tokio::test]
async fn test_executing_state_start_goes_to_completed() {
    let state = Arc::new(ExecutingState::<NoOpPJobExecutor>::default());
    let executor = crate::state_machine::executor::PJobExecutorAdapter::new(Arc::new(NoOpPJobExecutor));
    let next = state
        .on_event(
            StateChangeTrigger::Command(PJobCommand::Start),
            &executor,
            "pj-001",
        )
        .await
        .unwrap();
    assert_eq!(next.state_id(), PJobState::Completed);
}

#[tokio::test]
async fn test_paused_state_resume_goes_to_executing() {
    let state = Arc::new(PausedState::<NoOpPJobExecutor>::default());
    let executor = crate::state_machine::executor::PJobExecutorAdapter::new(Arc::new(NoOpPJobExecutor));
    let next = state
        .on_event(
            StateChangeTrigger::Command(PJobCommand::Resume),
            &executor,
            "pj-001",
        )
        .await
        .unwrap();
    assert_eq!(next.state_id(), PJobState::Executing);
}

#[tokio::test]
async fn test_terminal_state_rejects_all_commands() {
    let states: Vec<Arc<dyn State<Command = PJobCommand, Executor = crate::state_machine::executor::PJobExecutorAdapter<NoOpPJobExecutor>, Error = std::io::Error>>> = vec![
        Arc::new(CompletedState::<NoOpPJobExecutor>::default()),
        Arc::new(AbortedState::<NoOpPJobExecutor>::default()),
        Arc::new(CancelledState::<NoOpPJobExecutor>::default()),
        Arc::new(StoppedState::<NoOpPJobExecutor>::default()),
    ];

    let executor = crate::state_machine::executor::PJobExecutorAdapter::new(Arc::new(NoOpPJobExecutor));

    for state in states {
        let next = state
            .clone()
            .on_event(
                StateChangeTrigger::Command(PJobCommand::Start),
                &executor,
                "pj-001",
            )
            .await
            .unwrap();
        // Terminal states should return themselves
        assert_eq!(next.state_id(), state.state_id());
    }
}

// ─────────────────────────────────────────────
// Executor adapter tests
// ─────────────────────────────────────────────

#[tokio::test]
async fn test_executor_adapter_calls_user_executor() {
    let counting = CountingPJobExecutor::new();
    let adapter = crate::state_machine::executor::PJobExecutorAdapter::new(Arc::new(counting.clone()));

    // Simulate a transition through multiple states
    let _ = adapter.executor.setup("pj-001").await.unwrap();
    let _ = adapter.executor.start("pj-001").await.unwrap();
    let _ = adapter.executor.pause("pj-001").await.unwrap();
    let _ = adapter.executor.resume("pj-001").await.unwrap();
    let _ = adapter.executor.stop("pj-001").await.unwrap();

    assert_eq!(counting.setup_count.load(Ordering::SeqCst), 1);
    assert_eq!(counting.start_count.load(Ordering::SeqCst), 1);
    assert_eq!(counting.pause_count.load(Ordering::SeqCst), 1);
    assert_eq!(counting.resume_count.load(Ordering::SeqCst), 1);
    assert_eq!(counting.stop_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_executor_adapter_setup_wait_for_carrier() {
    #[derive(Debug, Clone)]
    struct WaitForCarrierExecutor;

    #[async_trait]
    impl PJobExecutor for WaitForCarrierExecutor {
        type Error = std::io::Error;

        async fn setup(&self, _pjob_id: &str) -> Result<SetupResult, Self::Error> {
            Ok(SetupResult::WaitForCarrier)
        }

        async fn start(&self, _pjob_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        async fn pause(&self, _pjob_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        async fn resume(&self, _pjob_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        async fn stop(&self, _pjob_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        async fn abort(&self, _pjob_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        async fn complete(&self, _pjob_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        async fn cancel(&self, _pjob_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    let state = Arc::new(QueuedState::<WaitForCarrierExecutor>::default());
    let executor = crate::state_machine::executor::PJobExecutorAdapter::new(Arc::new(WaitForCarrierExecutor));
    let next = state
        .on_event(
            StateChangeTrigger::Command(PJobCommand::Start),
            &executor,
            "pj-001",
        )
        .await
        .unwrap();
    assert_eq!(next.state_id(), PJobState::WaitingForStart);
}

#[tokio::test]
async fn test_custom_error_type() {
    #[derive(Debug)]
    enum MyPJobError {
        RecipeNotFound(String),
        HardwareNotReady,
    }

    impl std::fmt::Display for MyPJobError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                MyPJobError::RecipeNotFound(id) => write!(f, "Recipe not found: {id}"),
                MyPJobError::HardwareNotReady => write!(f, "Hardware not ready"),
            }
        }
    }

    impl std::error::Error for MyPJobError {}

    #[derive(Debug, Clone)]
    struct FailingExecutor;

    #[async_trait]
    impl PJobExecutor for FailingExecutor {
        type Error = MyPJobError;

        async fn setup(&self, _pjob_id: &str,
        ) -> Result<SetupResult, Self::Error> {
            Err(MyPJobError::RecipeNotFound("recipe-001".to_string()))
        }

        async fn start(&self, _pjob_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        async fn pause(&self, _pjob_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        async fn resume(&self, _pjob_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        async fn stop(&self, _pjob_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        async fn abort(&self, _pjob_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        async fn complete(&self, _pjob_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
        async fn cancel(&self, _pjob_id: &str) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    let state = Arc::new(QueuedState::<FailingExecutor>::default());
    let executor = crate::state_machine::executor::PJobExecutorAdapter::new(Arc::new(FailingExecutor));
    let result = state
        .on_event(
            StateChangeTrigger::Command(PJobCommand::Start),
            &executor,
            "pj-001",
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Recipe not found"));
}

// ─────────────────────────────────────────────
// AsyncStateMachine integration tests
// ─────────────────────────────────────────────

#[tokio::test]
async fn test_async_state_machine_full_flow() {
    use semi_common::async_state_machine::AsyncStateMachine;

    let counting = CountingPJobExecutor::new();
    let executor = Arc::new(crate::state_machine::executor::PJobExecutorAdapter::new(Arc::new(counting.clone())));
    let initial: Arc<dyn State<Command = PJobCommand, Executor = crate::state_machine::executor::PJobExecutorAdapter<CountingPJobExecutor>, Error = std::io::Error>> =
        Arc::new(QueuedState::<CountingPJobExecutor>::default());

    let sm = AsyncStateMachine::new(
        "pj-001".to_string(),
        initial,
        executor,
    );

    // Queued -> Start -> Selected
    let state = sm.handle_command(PJobCommand::Start).await.unwrap();
    assert_eq!(state, PJobState::Selected);

    // Selected -> Start -> WaitingForStart
    let state = sm.handle_command(PJobCommand::Start).await.unwrap();
    assert_eq!(state, PJobState::WaitingForStart);

    // WaitingForStart -> Start -> Executing
    let state = sm.handle_command(PJobCommand::Start).await.unwrap();
    assert_eq!(state, PJobState::Executing);

    // Executing -> Pause -> Paused
    let state = sm.handle_command(PJobCommand::Pause).await.unwrap();
    assert_eq!(state, PJobState::Paused);

    // Paused -> Resume -> Executing
    let state = sm.handle_command(PJobCommand::Resume).await.unwrap();
    assert_eq!(state, PJobState::Executing);

    // Executing -> Start (complete) -> Completed
    let state = sm.handle_command(PJobCommand::Start).await.unwrap();
    assert_eq!(state, PJobState::Completed);

    // Verify executor was called
    assert_eq!(counting.setup_count.load(Ordering::SeqCst), 1);
    assert_eq!(counting.start_count.load(Ordering::SeqCst), 2); // WaitingForStart, Executing (Selected uses start too, but let's check actual)
    assert_eq!(counting.pause_count.load(Ordering::SeqCst), 1);
    assert_eq!(counting.resume_count.load(Ordering::SeqCst), 1);
    assert_eq!(counting.complete_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_async_state_machine_abort_from_executing() {
    use semi_common::async_state_machine::AsyncStateMachine;

    let counting = CountingPJobExecutor::new();
    let executor = Arc::new(crate::state_machine::executor::PJobExecutorAdapter::new(Arc::new(counting.clone())));
    let initial: Arc<dyn State<Command = PJobCommand, Executor = crate::state_machine::executor::PJobExecutorAdapter<CountingPJobExecutor>, Error = std::io::Error>> =
        Arc::new(ExecutingState::<CountingPJobExecutor>::default());

    let sm = AsyncStateMachine::new(
        "pj-001".to_string(),
        initial,
        executor,
    );

    let state = sm.handle_command(PJobCommand::Abort).await.unwrap();
    assert_eq!(state, PJobState::Aborted);
    assert_eq!(counting.abort_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_async_state_machine_cancel_from_queued() {
    use semi_common::async_state_machine::AsyncStateMachine;

    let counting = CountingPJobExecutor::new();
    let executor = Arc::new(crate::state_machine::executor::PJobExecutorAdapter::new(Arc::new(counting.clone())));
    let initial: Arc<dyn State<Command = PJobCommand, Executor = crate::state_machine::executor::PJobExecutorAdapter<CountingPJobExecutor>, Error = std::io::Error>> =
        Arc::new(QueuedState::<CountingPJobExecutor>::default());

    let sm = AsyncStateMachine::new(
        "pj-001".to_string(),
        initial,
        executor,
    );

    let state = sm.handle_command(PJobCommand::Cancel).await.unwrap();
    assert_eq!(state, PJobState::Cancelled);
    assert_eq!(counting.cancel_count.load(Ordering::SeqCst), 1);
}
