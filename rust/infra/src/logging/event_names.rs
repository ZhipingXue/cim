//! Event name constants for classification.
//!
//! These constants are used across all CIM services for consistent
//! event naming and filtering.

// State machine events
pub const SM_TRANSITION_ATTEMPT: &str = "sm.transition.attempt";
pub const SM_TRANSITION_SUCCESS: &str = "sm.transition.success";
pub const SM_TRANSITION_FAILURE: &str = "sm.transition.failure";
pub const SM_INVALID_TRIGGER: &str = "sm.invalid_trigger";
pub const SM_TERMINAL_REJECT: &str = "sm.terminal.reject";

// Executor events
pub const EXEC_SETUP: &str = "exec.setup";
pub const EXEC_START: &str = "exec.start";
pub const EXEC_PAUSE: &str = "exec.pause";
pub const EXEC_RESUME: &str = "exec.resume";
pub const EXEC_STOP: &str = "exec.stop";
pub const EXEC_ABORT: &str = "exec.abort";
pub const EXEC_COMPLETE: &str = "exec.complete";
pub const EXEC_CANCEL: &str = "exec.cancel";
pub const EXEC_ERROR: &str = "exec.error";

// Store events
pub const STORE_CREATE: &str = "store.create";
pub const STORE_CREATE_DUPLICATE: &str = "store.create.duplicate";
pub const STORE_GET: &str = "store.get";
pub const STORE_GET_NOT_FOUND: &str = "store.get.not_found";
pub const STORE_LIST: &str = "store.list";
pub const STORE_COMMAND: &str = "store.command";
pub const STORE_COMMAND_INVALID: &str = "store.command.invalid";
pub const STORE_COMMAND_NOT_FOUND: &str = "store.command.not_found";
pub const STORE_UPDATE: &str = "store.update";
pub const STORE_DELETE: &str = "store.delete";

// Service events
pub const SVC_REQUEST: &str = "svc.request";
pub const SVC_RESPONSE: &str = "svc.response";
pub const SVC_ERROR: &str = "svc.error";
pub const SVC_INITIALIZE: &str = "svc.initialize";
pub const SVC_SHUTDOWN: &str = "svc.shutdown";
