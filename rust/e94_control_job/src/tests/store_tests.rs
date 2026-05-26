use crate::domain::model::{CJobCommand, CJobState, ControlJob, MaterialDestination};
use crate::state_machine::executor::NoOpCJobExecutor;
use crate::store::CJobStore;
use semi_common::error::CimError;
use std::sync::Arc;

fn dummy_job(cjob_id: &str, state: CJobState) -> ControlJob {
    ControlJob {
        cjob_id: cjob_id.to_string(),
        state,
        pjob_ids: vec![],
        destination: MaterialDestination::Unspecified,
        destination_carrier_id: String::new(),
        created_at: None,
        started_at: None,
        completed_at: None,
    }
}

#[tokio::test]
async fn test_create_and_get() {
    let store = CJobStore::new(Arc::new(NoOpCJobExecutor));
    let job = dummy_job("cj-001", CJobState::Queued);
    store.create(job.clone()).await.unwrap();

    let found = store.get("cj-001").await.unwrap();
    assert_eq!(found.cjob_id, "cj-001");
    assert_eq!(found.state, CJobState::Queued);
}

#[tokio::test]
async fn test_duplicate_create_fails() {
    let store = CJobStore::new(Arc::new(NoOpCJobExecutor));
    let job = dummy_job("cj-001", CJobState::Queued);
    store.create(job.clone()).await.unwrap();

    let err = store.create(job).await.unwrap_err();
    assert!(matches!(err, CimError::AlreadyExists(_)));
}

#[tokio::test]
async fn test_valid_start_manual() {
    let store = CJobStore::new(Arc::new(NoOpCJobExecutor));
    let job = dummy_job("cj-001", CJobState::Queued);
    store.create(job).await.unwrap();

    let j = store.execute_command("cj-001", CJobCommand::Start).await.unwrap();
    assert_eq!(j.state, CJobState::Selected);

    let j = store.execute_command("cj-001", CJobCommand::Start).await.unwrap();
    assert_eq!(j.state, CJobState::WaitingForStart);

    let j = store.execute_command("cj-001", CJobCommand::Start).await.unwrap();
    assert_eq!(j.state, CJobState::Executing);
    assert!(j.started_at.is_some());
}

#[tokio::test]
async fn test_pause_resume() {
    let store = CJobStore::new(Arc::new(NoOpCJobExecutor));
    let job = dummy_job("cj-001", CJobState::Executing);
    store.create(job).await.unwrap();

    let j = store.execute_command("cj-001", CJobCommand::Pause).await.unwrap();
    assert_eq!(j.state, CJobState::Paused);

    let j = store.execute_command("cj-001", CJobCommand::Resume).await.unwrap();
    assert_eq!(j.state, CJobState::Executing);
}

#[tokio::test]
async fn test_stop_from_executing() {
    let store = CJobStore::new(Arc::new(NoOpCJobExecutor));
    let job = dummy_job("cj-001", CJobState::Executing);
    store.create(job).await.unwrap();

    let j = store.execute_command("cj-001", CJobCommand::Stop).await.unwrap();
    assert_eq!(j.state, CJobState::Stopped);
    assert!(j.completed_at.is_some());
}

#[tokio::test]
async fn test_cancel_from_queued() {
    let store = CJobStore::new(Arc::new(NoOpCJobExecutor));
    let job = dummy_job("cj-001", CJobState::Queued);
    store.create(job).await.unwrap();

    let j = store.execute_command("cj-001", CJobCommand::Cancel).await.unwrap();
    assert_eq!(j.state, CJobState::Cancelled);
    assert!(j.completed_at.is_some());
}

#[tokio::test]
async fn test_complete_from_executing() {
    let store = CJobStore::new(Arc::new(NoOpCJobExecutor));
    let job = dummy_job("cj-001", CJobState::Executing);
    store.create(job).await.unwrap();

    let j = store.execute_command("cj-001", CJobCommand::Start).await.unwrap();
    assert_eq!(j.state, CJobState::Completed);
    assert!(j.completed_at.is_some());
}

#[tokio::test]
async fn test_invalid_command_fails() {
    let store = CJobStore::new(Arc::new(NoOpCJobExecutor));
    let job = dummy_job("cj-001", CJobState::Queued);
    store.create(job).await.unwrap();

    let err = store.execute_command("cj-001", CJobCommand::Pause).await.unwrap_err();
    assert!(matches!(err, CimError::InvalidArgument(_)));
}

#[tokio::test]
async fn test_list_filter_by_state() {
    let store = CJobStore::new(Arc::new(NoOpCJobExecutor));
    store.create(dummy_job("cj-001", CJobState::Queued)).await.unwrap();
    store.create(dummy_job("cj-002", CJobState::Executing)).await.unwrap();
    store.create(dummy_job("cj-003", CJobState::Queued)).await.unwrap();

    let queued = store.list(Some(CJobState::Queued)).await;
    assert_eq!(queued.len(), 2);

    let executing = store.list(Some(CJobState::Executing)).await;
    assert_eq!(executing.len(), 1);

    let all = store.list(None).await;
    assert_eq!(all.len(), 3);
}
