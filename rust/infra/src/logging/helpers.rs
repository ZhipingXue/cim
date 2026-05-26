//! Structured logging helper functions.
//!
//! These functions wrap `tracing` macros with consistent `event_name` fields
//! for classification and filtering by custom tracing layers.

use tracing::{debug, error, info, warn};

use super::event_names;

// ─────────────────────────────────────────────
// State machine logging
// ─────────────────────────────────────────────

/// Log a state transition attempt.
#[inline]
pub fn log_transition_attempt(sm_id: &str, from: &str, trigger: &str) {
    info!(
        event_name = event_names::SM_TRANSITION_ATTEMPT,
        sm_id = %sm_id,
        from_state = %from,
        trigger = %trigger,
        "State transition attempt"
    );
}

/// Log a successful state transition.
#[inline]
pub fn log_transition_success(sm_id: &str, from: &str, to: &str, trigger: &str) {
    info!(
        event_name = event_names::SM_TRANSITION_SUCCESS,
        sm_id = %sm_id,
        from_state = %from,
        to_state = %to,
        trigger = %trigger,
        "State transition succeeded"
    );
}

/// Log a failed state transition.
#[inline]
pub fn log_transition_failure(sm_id: &str, from: &str, trigger: &str, error: &str) {
    error!(
        event_name = event_names::SM_TRANSITION_FAILURE,
        sm_id = %sm_id,
        from_state = %from,
        trigger = %trigger,
        error = %error,
        "State transition failed"
    );
}

/// Log an invalid trigger for a state.
#[inline]
pub fn log_invalid_trigger(sm_id: &str, state: &str, trigger: &str) {
    warn!(
        event_name = event_names::SM_INVALID_TRIGGER,
        sm_id = %sm_id,
        state = %state,
        trigger = %trigger,
        "Invalid trigger for current state"
    );
}

/// Log a terminal state rejecting a command.
#[inline]
pub fn log_terminal_reject(sm_id: &str, state: &str, trigger: &str) {
    warn!(
        event_name = event_names::SM_TERMINAL_REJECT,
        sm_id = %sm_id,
        state = %state,
        trigger = %trigger,
        "Terminal state rejected command"
    );
}

// ─────────────────────────────────────────────
// Executor logging
// ─────────────────────────────────────────────

/// Log executor method entry.
#[inline]
pub fn log_exec_entry(event_name: &str, entity_id: &str, method: &str) {
    info!(
        event_name = %event_name,
        entity_id = %entity_id,
        method = %method,
        "Executor method called"
    );
}

/// Log executor error.
#[inline]
pub fn log_exec_error(entity_id: &str, method: &str, error: &str) {
    error!(
        event_name = event_names::EXEC_ERROR,
        entity_id = %entity_id,
        method = %method,
        error = %error,
        "Executor method failed"
    );
}

// ─────────────────────────────────────────────
// Store logging
// ─────────────────────────────────────────────

/// Log store create.
#[inline]
pub fn log_store_create(entity_id: &str, entity_type: &str, state: &str) {
    info!(
        event_name = event_names::STORE_CREATE,
        entity_id = %entity_id,
        entity_type = %entity_type,
        initial_state = %state,
        "Entity created in store"
    );
}

/// Log duplicate create attempt.
#[inline]
pub fn log_store_create_duplicate(entity_id: &str, entity_type: &str) {
    warn!(
        event_name = event_names::STORE_CREATE_DUPLICATE,
        entity_id = %entity_id,
        entity_type = %entity_type,
        "Duplicate entity creation attempt"
    );
}

/// Log store get.
#[inline]
pub fn log_store_get(entity_id: &str, entity_type: &str, found: bool) {
    if found {
        debug!(
            event_name = event_names::STORE_GET,
            entity_id = %entity_id,
            entity_type = %entity_type,
            "Entity retrieved from store"
        );
    } else {
        warn!(
            event_name = event_names::STORE_GET_NOT_FOUND,
            entity_id = %entity_id,
            entity_type = %entity_type,
            "Entity not found in store"
        );
    }
}

/// Log store command execution.
#[inline]
pub fn log_store_command(entity_id: &str, entity_type: &str, command: &str, from_state: &str, to_state: &str) {
    info!(
        event_name = event_names::STORE_COMMAND,
        entity_id = %entity_id,
        entity_type = %entity_type,
        command = %command,
        from_state = %from_state,
        to_state = %to_state,
        "Entity command executed"
    );
}

/// Log invalid store command.
#[inline]
pub fn log_store_command_invalid(entity_id: &str, entity_type: &str, command: &str, state: &str) {
    warn!(
        event_name = event_names::STORE_COMMAND_INVALID,
        entity_id = %entity_id,
        entity_type = %entity_type,
        command = %command,
        state = %state,
        "Invalid command for current state"
    );
}

/// Log command on non-existent entity.
#[inline]
pub fn log_store_command_not_found(entity_id: &str, entity_type: &str, command: &str) {
    warn!(
        event_name = event_names::STORE_COMMAND_NOT_FOUND,
        entity_id = %entity_id,
        entity_type = %entity_type,
        command = %command,
        "Command on non-existent entity"
    );
}

/// Log store update.
#[inline]
pub fn log_store_update(entity_id: &str, entity_type: &str, fields: &str) {
    info!(
        event_name = event_names::STORE_UPDATE,
        entity_id = %entity_id,
        entity_type = %entity_type,
        fields = %fields,
        "Entity updated in store"
    );
}

/// Log store delete.
#[inline]
pub fn log_store_delete(entity_id: &str, entity_type: &str) {
    info!(
        event_name = event_names::STORE_DELETE,
        entity_id = %entity_id,
        entity_type = %entity_type,
        "Entity deleted from store"
    );
}

// ─────────────────────────────────────────────
// Service logging
// ─────────────────────────────────────────────

/// Log gRPC request.
#[inline]
pub fn log_svc_request(method: &str, entity_id: Option<&str>) {
    if let Some(id) = entity_id {
        info!(
            event_name = event_names::SVC_REQUEST,
            method = %method,
            entity_id = %id,
            "gRPC request received"
        );
    } else {
        info!(
            event_name = event_names::SVC_REQUEST,
            method = %method,
            "gRPC request received"
        );
    }
}

/// Log gRPC response.
#[inline]
pub fn log_svc_response(method: &str, entity_id: Option<&str>, success: bool) {
    if let Some(id) = entity_id {
        info!(
            event_name = event_names::SVC_RESPONSE,
            method = %method,
            entity_id = %id,
            success = %success,
            "gRPC response sent"
        );
    } else {
        info!(
            event_name = event_names::SVC_RESPONSE,
            method = %method,
            success = %success,
            "gRPC response sent"
        );
    }
}

/// Log service error.
#[inline]
pub fn log_svc_error(method: &str, entity_id: Option<&str>, error: &str) {
    if let Some(id) = entity_id {
        error!(
            event_name = event_names::SVC_ERROR,
            method = %method,
            entity_id = %id,
            error = %error,
            "gRPC request failed"
        );
    } else {
        error!(
            event_name = event_names::SVC_ERROR,
            method = %method,
            error = %error,
            "gRPC request failed"
        );
    }
}

/// Log service lifecycle.
#[inline]
pub fn log_svc_lifecycle(event_name: &str, service_type: &str) {
    info!(
        event_name = %event_name,
        service_type = %service_type,
        "Service lifecycle event"
    );
}
