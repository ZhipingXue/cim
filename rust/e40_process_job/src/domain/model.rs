use std::time::SystemTime;

// ─────────────────────────────────────────────
// Domain types — pure Rust, no proto dependency
// ─────────────────────────────────────────────

/// E40 ProcessJob state (domain enum, not proto-generated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PJobState {
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

/// E40 ProcessJob command (domain enum, not proto-generated).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PJobCommand {
    Start,
    Pause,
    Resume,
    Abort,
    Cancel,
    Stop,
}

/// Recipe method (domain enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecipeMethod {
    #[default]
    Unspecified,
    RecipeOnly,
    RecipeWithVariableTuning,
}

/// Recipe variable (domain struct).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeVariable {
    pub name: String,
    pub value: String,
}

/// Recipe (domain struct).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipe {
    pub recipe_name: String,
    pub recipe_body: String,
}

/// Material (domain struct).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Material {
    pub substrate_id: String,
    pub process_recipe: String,
}

/// ProcessJob — the domain entity.
///
/// This is the canonical representation used by the state machine,
/// store, and tests. Proto types are only used at the gRPC boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessJob {
    pub pjob_id: String,
    pub state: PJobState,
    pub materials: Vec<Material>,
    pub recipe: Option<Recipe>,
    pub cjob_id: String,
    pub created_at: Option<SystemTime>,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub pr_process_start: bool,   // true = auto start
    pub pr_recipe_method: RecipeMethod,
    pub recipe_variables: Vec<RecipeVariable>,
}

impl ProcessJob {
    pub fn new(pjob_id: impl Into<String>) -> Self {
        Self {
            pjob_id: pjob_id.into(),
            state: PJobState::Queued,
            materials: vec![],
            recipe: None,
            cjob_id: String::new(),
            created_at: Some(SystemTime::now()),
            started_at: None,
            completed_at: None,
            pr_process_start: false,
            pr_recipe_method: RecipeMethod::Unspecified,
            recipe_variables: vec![],
        }
    }

    /// Returns true if this state is terminal (no further transitions).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.state,
            PJobState::Completed | PJobState::Aborted | PJobState::Cancelled | PJobState::Stopped
        )
    }
}
