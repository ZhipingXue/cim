use std::time::SystemTime;

// ─────────────────────────────────────────────
// Domain types — pure Rust, no proto dependency
// ─────────────────────────────────────────────

/// E94 ControlJob state (domain enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CJobState {
    Queued,
    Selected,
    WaitingForStart,
    Executing,
    Paused,
    Completed,
    Aborted,
    Cancelled,
    Stopped,
}

/// E94 ControlJob command (domain enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CJobCommand {
    Start,
    Pause,
    Resume,
    Abort,
    Cancel,
    Stop,
}

/// Material destination after processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaterialDestination {
    #[default]
    Unspecified,
    SourceCarrier,
    DestinationCarrier,
    Sorting,
}

/// ControlJob — the domain entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlJob {
    pub cjob_id: String,
    pub state: CJobState,
    pub pjob_ids: Vec<String>,
    pub destination: MaterialDestination,
    pub destination_carrier_id: String,
    pub created_at: Option<SystemTime>,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
}

impl ControlJob {
    pub fn new(cjob_id: impl Into<String>) -> Self {
        Self {
            cjob_id: cjob_id.into(),
            state: CJobState::Queued,
            pjob_ids: vec![],
            destination: MaterialDestination::Unspecified,
            destination_carrier_id: String::new(),
            created_at: Some(SystemTime::now()),
            started_at: None,
            completed_at: None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.state,
            CJobState::Completed | CJobState::Aborted | CJobState::Cancelled | CJobState::Stopped
        )
    }
}
