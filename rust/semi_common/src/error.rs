use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CimError {
    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Entity already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Alarm: {code} - {description}")]
    Alarm { code: String, description: String },
}

pub type CimResult<T> = Result<T, CimError>;
