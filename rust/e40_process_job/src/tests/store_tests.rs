use crate::domain::model::{PJobCommand, PJobState, ProcessJob, RecipeMethod};
use crate::state_machine::executor::NoOpPJobExecutor;
use crate::store::PJobStore;
use semi_common::error::CimError;
use std::sync::Arc;

fn dummy_job(pjob_id: &str, state: PJobState) -> ProcessJob {
    ProcessJob {
        pjob_id: pjob_id.to_string(),
        state,
        materials: vec![],
        recipe: None,
        cjob_id: "".to_string(),
        created_at: None,
        started_at: None,
        completed_at: None,
        pr_process_start: false,
        pr_recipe_method: RecipeMethod::Unspecified,
        recipe_variables: vec![],
    }
}

#[tokio::test]
async fn test_create_and_get() {
    let store = PJobStore::new(Arc::new(NoOpPJobExecutor));
    let job = dummy_job("pj-001", PJobState::Queued);
    store.create(job.clone()).await.unwrap();

    let found = store.get("pj-001").await.unwrap();
    assert_eq!(found.pjob_id, "pj-001");
    assert_eq!(found.state, PJobState::Queued);
}

#[tokio::test]
async fn test_duplicate_create_fails() {
    let store = PJobStore::new(Arc::new(NoOpPJobExecutor));
    let job = dummy_job("pj-001", PJobState::Queued);
    store.create(job.clone()).await.unwrap();

    let err = store.create(job).await.unwrap_err();
    assert!(matches!(err, CimError::AlreadyExists(_)));
}

#[tokio::test]
async fn test_valid_start_manual() {
    let store = PJobStore::new(Arc::new(NoOpPJobExecutor));
    let mut job = dummy_job("pj-001", PJobState::Queued);
    job.pr_process_start = false;
    store.create(job).await.unwrap();

    // START from QUEUED (manual) → SELECTED
    let j = store.execute_command("pj-001", PJobCommand::Start).await.unwrap();
    assert_eq!(j.state, PJobState::Selected);
    assert!(j.started_at.is_none());

    // START from SELECTED (manual) → WAITING_FOR_START
    let j = store.execute_command("pj-001", PJobCommand::Start).await.unwrap();
    assert_eq!(j.state, PJobState::WaitingForStart);
    assert!(j.started_at.is_none());

    // START from WAITING_FOR_START → EXECUTING
    let j = store.execute_command("pj-001", PJobCommand::Start).await.unwrap();
    assert_eq!(j.state, PJobState::Executing);
    assert!(j.started_at.is_some());
}

#[tokio::test]
async fn test_valid_start_auto() {
    let store = PJobStore::new(Arc::new(NoOpPJobExecutor));
    let mut job = dummy_job("pj-001", PJobState::Queued);
    job.pr_process_start = true;
    store.create(job).await.unwrap();

    // START from QUEUED (auto) → SELECTED (setup decides, not pr_process_start)
    // With NoOpPJobExecutor, setup returns Ready → Selected
    let j = store.execute_command("pj-001", PJobCommand::Start).await.unwrap();
    assert_eq!(j.state, PJobState::Selected);
}

#[tokio::test]
async fn test_pause_resume() {
    let store = PJobStore::new(Arc::new(NoOpPJobExecutor));
    let job = dummy_job("pj-001", PJobState::Executing);
    store.create(job).await.unwrap();

    // PAUSE
    let j = store.execute_command("pj-001", PJobCommand::Pause).await.unwrap();
    assert_eq!(j.state, PJobState::Paused);

    // RESUME
    let j = store.execute_command("pj-001", PJobCommand::Resume).await.unwrap();
    assert_eq!(j.state, PJobState::Executing);
}

#[tokio::test]
async fn test_stop_from_executing() {
    let store = PJobStore::new(Arc::new(NoOpPJobExecutor));
    let job = dummy_job("pj-001", PJobState::Executing);
    store.create(job).await.unwrap();

    let j = store.execute_command("pj-001", PJobCommand::Stop).await.unwrap();
    assert_eq!(j.state, PJobState::Stopped);
    assert!(j.completed_at.is_some());
}

#[tokio::test]
async fn test_abort_from_executing() {
    let store = PJobStore::new(Arc::new(NoOpPJobExecutor));
    let job = dummy_job("pj-001", PJobState::Executing);
    store.create(job).await.unwrap();

    let j = store.execute_command("pj-001", PJobCommand::Abort).await.unwrap();
    assert_eq!(j.state, PJobState::Aborted);
    assert!(j.completed_at.is_some());
}

#[tokio::test]
async fn test_cancel_from_queued() {
    let store = PJobStore::new(Arc::new(NoOpPJobExecutor));
    let job = dummy_job("pj-001", PJobState::Queued);
    store.create(job).await.unwrap();

    let j = store.execute_command("pj-001", PJobCommand::Cancel).await.unwrap();
    assert_eq!(j.state, PJobState::Cancelled);
    assert!(j.completed_at.is_some());
}

#[tokio::test]
async fn test_complete_from_executing() {
    let store = PJobStore::new(Arc::new(NoOpPJobExecutor));
    let job = dummy_job("pj-001", PJobState::Executing);
    store.create(job).await.unwrap();

    // START from EXECUTING signals completion
    let j = store.execute_command("pj-001", PJobCommand::Start).await.unwrap();
    assert_eq!(j.state, PJobState::Completed);
    assert!(j.completed_at.is_some());
}

#[tokio::test]
async fn test_invalid_command_fails() {
    let store = PJobStore::new(Arc::new(NoOpPJobExecutor));
    let job = dummy_job("pj-001", PJobState::Queued);
    store.create(job).await.unwrap();

    // PAUSE from QUEUED is invalid
    let err = store.execute_command("pj-001", PJobCommand::Pause).await.unwrap_err();
    assert!(matches!(err, CimError::InvalidArgument(_)));
}

#[tokio::test]
async fn test_terminal_state_rejects_commands() {
    let store = PJobStore::new(Arc::new(NoOpPJobExecutor));
    let job = dummy_job("pj-001", PJobState::Completed);
    store.create(job).await.unwrap();

    // Any command from COMPLETED is invalid
    let err = store.execute_command("pj-001", PJobCommand::Start).await.unwrap_err();
    assert!(matches!(err, CimError::InvalidArgument(_)));
}

#[tokio::test]
async fn test_list_filter_by_state() {
    let store = PJobStore::new(Arc::new(NoOpPJobExecutor));
    store.create(dummy_job("pj-001", PJobState::Queued)).await.unwrap();
    store.create(dummy_job("pj-002", PJobState::Executing)).await.unwrap();
    store.create(dummy_job("pj-003", PJobState::Queued)).await.unwrap();

    let queued = store.list(Some(PJobState::Queued), "").await;
    assert_eq!(queued.len(), 2);

    let executing = store.list(Some(PJobState::Executing), "").await;
    assert_eq!(executing.len(), 1);

    let all = store.list(None, "").await;
    assert_eq!(all.len(), 3);
}

#[tokio::test]
async fn test_list_filter_by_cjob_id() {
    let store = PJobStore::new(Arc::new(NoOpPJobExecutor));
    let mut job1 = dummy_job("pj-001", PJobState::Queued);
    job1.cjob_id = "cj-001".to_string();
    let mut job2 = dummy_job("pj-002", PJobState::Queued);
    job2.cjob_id = "cj-002".to_string();
    store.create(job1).await.unwrap();
    store.create(job2).await.unwrap();

    let cj1 = store.list(None, "cj-001").await;
    assert_eq!(cj1.len(), 1);
    assert_eq!(cj1[0].pjob_id, "pj-001");
}
