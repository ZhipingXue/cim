use std::time::SystemTime;

use crate::domain::model;
use crate::pb;

// ─────────────────────────────────────────────
// Proto → Domain conversions
// ─────────────────────────────────────────────

pub fn pjob_state_from_proto(state: i32) -> Option<model::PJobState> {
    use pb::e40::PJobState::*;
    match state {
        x if x == PjobStateQueued as i32 => Some(model::PJobState::Queued),
        x if x == PjobStateSelected as i32 => Some(model::PJobState::Selected),
        x if x == PjobStateWaitingForStart as i32 => Some(model::PJobState::WaitingForStart),
        x if x == PjobStateExecuting as i32 => Some(model::PJobState::Executing),
        x if x == PjobStatePaused as i32 => Some(model::PJobState::Paused),
        x if x == PjobStateCompleted as i32 => Some(model::PJobState::Completed),
        x if x == PjobStateAborted as i32 => Some(model::PJobState::Aborted),
        x if x == PjobStateCancelled as i32 => Some(model::PJobState::Cancelled),
        x if x == PjobStateStopped as i32 => Some(model::PJobState::Stopped),
        _ => None,
    }
}

pub fn pjob_state_to_proto(state: model::PJobState) -> i32 {
    use pb::e40::PJobState::*;
    match state {
        model::PJobState::Queued => PjobStateQueued as i32,
        model::PJobState::Selected => PjobStateSelected as i32,
        model::PJobState::WaitingForStart => PjobStateWaitingForStart as i32,
        model::PJobState::Executing => PjobStateExecuting as i32,
        model::PJobState::Paused => PjobStatePaused as i32,
        model::PJobState::Completed => PjobStateCompleted as i32,
        model::PJobState::Aborted => PjobStateAborted as i32,
        model::PJobState::Cancelled => PjobStateCancelled as i32,
        model::PJobState::Stopped => PjobStateStopped as i32,
    }
}

pub fn pjob_command_from_proto(cmd: i32) -> Option<model::PJobCommand> {
    use pb::e40::PJobCommand::*;
    match cmd {
        x if x == PjobCommandStart as i32 => Some(model::PJobCommand::Start),
        x if x == PjobCommandPause as i32 => Some(model::PJobCommand::Pause),
        x if x == PjobCommandResume as i32 => Some(model::PJobCommand::Resume),
        x if x == PjobCommandAbort as i32 => Some(model::PJobCommand::Abort),
        x if x == PjobCommandCancel as i32 => Some(model::PJobCommand::Cancel),
        x if x == PjobCommandStop as i32 => Some(model::PJobCommand::Stop),
        _ => None,
    }
}

pub fn pjob_command_to_proto(cmd: model::PJobCommand) -> i32 {
    use pb::e40::PJobCommand::*;
    match cmd {
        model::PJobCommand::Start => PjobCommandStart as i32,
        model::PJobCommand::Pause => PjobCommandPause as i32,
        model::PJobCommand::Resume => PjobCommandResume as i32,
        model::PJobCommand::Abort => PjobCommandAbort as i32,
        model::PJobCommand::Cancel => PjobCommandCancel as i32,
        model::PJobCommand::Stop => PjobCommandStop as i32,
    }
}

pub fn recipe_method_from_proto(method: i32) -> model::RecipeMethod {
    use pb::e40::RecipeMethod::*;
    match method {
        x if x == RecipeOnly as i32 => model::RecipeMethod::RecipeOnly,
        x if x == RecipeWithVariableTuning as i32 => {
            model::RecipeMethod::RecipeWithVariableTuning
        }
        _ => model::RecipeMethod::Unspecified,
    }
}

pub fn recipe_method_to_proto(method: model::RecipeMethod) -> i32 {
    use pb::e40::RecipeMethod::*;
    match method {
        model::RecipeMethod::Unspecified => Unspecified as i32,
        model::RecipeMethod::RecipeOnly => RecipeOnly as i32,
        model::RecipeMethod::RecipeWithVariableTuning => {
            RecipeWithVariableTuning as i32
        }
    }
}

pub fn recipe_from_proto(r: Option<pb::e40::Recipe>) -> Option<model::Recipe> {
    r.map(|r| model::Recipe {
        recipe_name: r.recipe_name,
        recipe_body: r.recipe_body,
    })
}

pub fn recipe_to_proto(r: Option<model::Recipe>) -> Option<pb::e40::Recipe> {
    r.map(|r| pb::e40::Recipe {
        recipe_name: r.recipe_name,
        recipe_body: r.recipe_body,
    })
}

pub fn material_from_proto(m: pb::e40::Material) -> model::Material {
    model::Material {
        substrate_id: m
            .substrate
            .map(|s| format!("{}:{}:{}", s.carrier_id, s.slot_number, s.substrate_id))
            .unwrap_or_default(),
        process_recipe: m.process_recipe,
    }
}

pub fn material_to_proto(m: model::Material) -> pb::e40::Material {
    pb::e40::Material {
        substrate: {
            let parts: Vec<&str> = m.substrate_id.splitn(3, ':').collect();
            if parts.len() == 3 {
                Some(pb::common::SubstrateId {
                    carrier_id: parts[0].to_string(),
                    slot_number: parts[1].parse().unwrap_or(0),
                    substrate_id: parts[2].to_string(),
                })
            } else {
                Some(pb::common::SubstrateId {
                    carrier_id: m.substrate_id.clone(),
                    slot_number: 0,
                    substrate_id: String::new(),
                })
            }
        },
        process_recipe: m.process_recipe,
    }
}

pub fn recipe_variable_from_proto(v: pb::e40::RecipeVariable) -> model::RecipeVariable {
    model::RecipeVariable {
        name: v.name,
        value: v.value,
    }
}

pub fn recipe_variable_to_proto(v: model::RecipeVariable) -> pb::e40::RecipeVariable {
    pb::e40::RecipeVariable {
        name: v.name,
        value: v.value,
    }
}

pub fn process_job_from_proto(job: pb::e40::ProcessJob) -> Option<model::ProcessJob> {
    Some(model::ProcessJob {
        pjob_id: job.pjob_id,
        state: pjob_state_from_proto(job.state)?,
        materials: job.materials.into_iter().map(material_from_proto).collect(),
        recipe: recipe_from_proto(job.recipe),
        cjob_id: job.cjob_id,
        created_at: job.created_at.and_then(|t| {
            SystemTime::UNIX_EPOCH
                .checked_add(std::time::Duration::from_secs(t.seconds as u64))
                .and_then(|s| s.checked_add(std::time::Duration::from_nanos(t.nanos as u64)))
        }),
        started_at: job.started_at.and_then(|t| {
            SystemTime::UNIX_EPOCH
                .checked_add(std::time::Duration::from_secs(t.seconds as u64))
                .and_then(|s| s.checked_add(std::time::Duration::from_nanos(t.nanos as u64)))
        }),
        completed_at: job.completed_at.and_then(|t| {
            SystemTime::UNIX_EPOCH
                .checked_add(std::time::Duration::from_secs(t.seconds as u64))
                .and_then(|s| s.checked_add(std::time::Duration::from_nanos(t.nanos as u64)))
        }),
        pr_process_start: job.pr_process_start,
        pr_recipe_method: recipe_method_from_proto(job.pr_recipe_method),
        recipe_variables: job
            .recipe_variables
            .into_iter()
            .map(recipe_variable_from_proto)
            .collect(),
    })
}

pub fn process_job_to_proto(job: model::ProcessJob) -> pb::e40::ProcessJob {
    use ::prost_types::Timestamp;
    pb::e40::ProcessJob {
        pjob_id: job.pjob_id,
        state: pjob_state_to_proto(job.state),
        materials: job.materials.into_iter().map(material_to_proto).collect(),
        recipe: recipe_to_proto(job.recipe),
        cjob_id: job.cjob_id,
        created_at: job.created_at.map(|t| {
            let dur = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
            Timestamp {
                seconds: dur.as_secs() as i64,
                nanos: dur.subsec_nanos() as i32,
            }
        }),
        started_at: job.started_at.map(|t| {
            let dur = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
            Timestamp {
                seconds: dur.as_secs() as i64,
                nanos: dur.subsec_nanos() as i32,
            }
        }),
        completed_at: job.completed_at.map(|t| {
            let dur = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
            Timestamp {
                seconds: dur.as_secs() as i64,
                nanos: dur.subsec_nanos() as i32,
            }
        }),
        pr_process_start: job.pr_process_start,
        pr_recipe_method: recipe_method_to_proto(job.pr_recipe_method),
        recipe_variables: job
            .recipe_variables
            .into_iter()
            .map(recipe_variable_to_proto)
            .collect(),
    }
}
