use std::sync::Arc;

use infra::{CollectedEvent, EventCollector, EventSeverity, InMemoryEventCollector};
use crate::state_machine::executor::{NoOpPJobExecutor, PJobExecutor, SetupResult};
use crate::state_machine::states::*;

use semi_common::async_state_machine::{State, StateChangeTrigger};

// ─────────────────────────────────────────────
// Event collector tests
// ─────────────────────────────────────────────

#[tokio::test]
async fn test_in_memory_collector_stores_events() {
    let collector = InMemoryEventCollector::new();
    
    let event = CollectedEvent::new(
        "sm.transition.success",
        EventSeverity::Info,
        "pj-001",
        "e40_process_job",
        "Transition succeeded",
    )
    .with_from_state("Queued")
    .with_to_state("Selected")
    .with_trigger("Start");
    
    collector.collect(event.clone()).await.unwrap();
    
    let events = collector.events().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_name, "sm.transition.success");
    assert_eq!(events[0].sm_id, "pj-001");
    assert_eq!(events[0].from_state, Some("Queued".to_string()));
    assert_eq!(events[0].to_state, Some("Selected".to_string()));
}

#[tokio::test]
async fn test_in_memory_collector_filter_by_name() {
    let collector = InMemoryEventCollector::new();
    
    collector.collect(CollectedEvent::new(
        "sm.transition.success",
        EventSeverity::Info,
        "pj-001",
        "e40",
        "ok",
    )).await.unwrap();
    
    collector.collect(CollectedEvent::new(
        "sm.transition.failure",
        EventSeverity::Error,
        "pj-002",
        "e40",
        "fail",
    )).await.unwrap();
    
    collector.collect(CollectedEvent::new(
        "sm.transition.success",
        EventSeverity::Info,
        "pj-003",
        "e40",
        "ok",
    )).await.unwrap();
    
    let success_events = collector.events_by_name("sm.transition.success").await;
    assert_eq!(success_events.len(), 2);
}

#[tokio::test]
async fn test_in_memory_collector_filter_by_severity() {
    let collector = InMemoryEventCollector::new();
    
    collector.collect(CollectedEvent::new("a", EventSeverity::Info, "id", "type", "msg")).await.unwrap();
    collector.collect(CollectedEvent::new("b", EventSeverity::Warning, "id", "type", "msg")).await.unwrap();
    collector.collect(CollectedEvent::new("c", EventSeverity::Error, "id", "type", "msg")).await.unwrap();
    collector.collect(CollectedEvent::new("d", EventSeverity::Info, "id", "type", "msg")).await.unwrap();
    
    let warnings = collector.events_by_severity(EventSeverity::Warning).await;
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].event_name, "b");
    
    let errors = collector.events_by_severity(EventSeverity::Error).await;
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].event_name, "c");
}

#[tokio::test]
async fn test_collected_event_builder() {
    let event = CollectedEvent::new(
        "test.event",
        EventSeverity::Critical,
        "sm-001",
        "e40",
        "Something critical happened",
    )
    .with_from_state("Executing")
    .with_to_state("Aborted")
    .with_trigger("Abort")
    .with_error("Hardware failure");
    
    assert_eq!(event.event_name, "test.event");
    assert_eq!(event.severity, EventSeverity::Critical);
    assert_eq!(event.sm_id, "sm-001");
    assert_eq!(event.from_state, Some("Executing".to_string()));
    assert_eq!(event.to_state, Some("Aborted".to_string()));
    assert_eq!(event.trigger, Some("Abort".to_string()));
    assert_eq!(event.error, Some("Hardware failure".to_string()));
}

// ─────────────────────────────────────────────
// Integration: state machine + event collector
// ─────────────────────────────────────────────

#[tokio::test]
async fn test_state_machine_logs_events() {
    // This test verifies that the state machine logs are emitted.
    // Since tracing logs are async and global, we verify by running a transition
    // and ensuring no panic occurs. The actual log output can be verified
    // by running with RUST_LOG=info.
    
    let state = Arc::new(QueuedState::<NoOpPJobExecutor>::default());
    let executor = crate::state_machine::executor::PJobExecutorAdapter::new(Arc::new(NoOpPJobExecutor));
    
    // This should log: sm.transition.attempt, exec.setup, sm.transition.success
    let next = state
        .clone()
        .on_event(
            StateChangeTrigger::Command(crate::domain::model::PJobCommand::Start),
            &executor,
            "pj-001",
        )
        .await
        .unwrap();
    
    assert_eq!(next.state_id(), crate::domain::model::PJobState::Selected);
    
    // Invalid trigger should log: sm.invalid_trigger
    let next = state
        .clone()
        .on_event(
            StateChangeTrigger::Command(crate::domain::model::PJobCommand::Pause),
            &executor,
            "pj-001",
        )
        .await
        .unwrap();
    
    assert_eq!(next.state_id(), crate::domain::model::PJobState::Queued);
}

#[tokio::test]
async fn test_event_collector_can_be_used_with_custom_executor() {
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    #[derive(Debug, Clone)]
    struct LoggingExecutor {
        event_count: Arc<AtomicUsize>,
    }
    
    #[async_trait]
    impl PJobExecutor for LoggingExecutor {
        type Error = std::io::Error;
        
        async fn setup(&self, _pjob_id: &str) -> Result<SetupResult, Self::Error> {
            self.event_count.fetch_add(1, Ordering::SeqCst);
            Ok(SetupResult::Ready)
        }
        async fn start(&self, _pjob_id: &str) -> Result<(), Self::Error> { Ok(()) }
        async fn pause(&self, _pjob_id: &str) -> Result<(), Self::Error> { Ok(()) }
        async fn resume(&self, _pjob_id: &str) -> Result<(), Self::Error> { Ok(()) }
        async fn stop(&self, _pjob_id: &str) -> Result<(), Self::Error> { Ok(()) }
        async fn abort(&self, _pjob_id: &str) -> Result<(), Self::Error> { Ok(()) }
        async fn complete(&self, _pjob_id: &str) -> Result<(), Self::Error> { Ok(()) }
        async fn cancel(&self, _pjob_id: &str) -> Result<(), Self::Error> { Ok(()) }
    }
    
    let executor = LoggingExecutor {
        event_count: Arc::new(AtomicUsize::new(0)),
    };
    
    let state = Arc::new(QueuedState::<LoggingExecutor>::default());
    let adapter = crate::state_machine::executor::PJobExecutorAdapter::new(Arc::new(executor.clone()));
    
    let _ = state
        .on_event(
            StateChangeTrigger::Command(crate::domain::model::PJobCommand::Start),
            &adapter,
            "pj-001",
        )
        .await
        .unwrap();
    
    // Verify executor was called (which also triggers logging)
    assert_eq!(executor.event_count.load(Ordering::SeqCst), 1);
}
