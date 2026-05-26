use std::time::SystemTime;

use crate::domain::model;
use crate::pb;

// ─────────────────────────────────────────────
// Proto → Domain conversions
// ─────────────────────────────────────────────

pub fn cjob_state_from_proto(state: i32) -> Option<model::CJobState> {
    use pb::e94::CJobState::*;
    match state {
        x if x == CjobStateQueued as i32 => Some(model::CJobState::Queued),
        x if x == CjobStateSelected as i32 => Some(model::CJobState::Selected),
        x if x == CjobStateWaitingForStart as i32 => Some(model::CJobState::WaitingForStart),
        x if x == CjobStateExecuting as i32 => Some(model::CJobState::Executing),
        x if x == CjobStatePaused as i32 => Some(model::CJobState::Paused),
        x if x == CjobStateCompleted as i32 => Some(model::CJobState::Completed),
        x if x == CjobStateAborted as i32 => Some(model::CJobState::Aborted),
        x if x == CjobStateCancelled as i32 => Some(model::CJobState::Cancelled),
        x if x == CjobStateStopped as i32 => Some(model::CJobState::Stopped),
        _ => None,
    }
}

pub fn cjob_state_to_proto(state: model::CJobState) -> i32 {
    use pb::e94::CJobState::*;
    match state {
        model::CJobState::Queued => CjobStateQueued as i32,
        model::CJobState::Selected => CjobStateSelected as i32,
        model::CJobState::WaitingForStart => CjobStateWaitingForStart as i32,
        model::CJobState::Executing => CjobStateExecuting as i32,
        model::CJobState::Paused => CjobStatePaused as i32,
        model::CJobState::Completed => CjobStateCompleted as i32,
        model::CJobState::Aborted => CjobStateAborted as i32,
        model::CJobState::Cancelled => CjobStateCancelled as i32,
        model::CJobState::Stopped => CjobStateStopped as i32,
    }
}

pub fn cjob_command_from_proto(cmd: i32) -> Option<model::CJobCommand> {
    use pb::e94::CJobCommand::*;
    match cmd {
        x if x == CjobCommandStart as i32 => Some(model::CJobCommand::Start),
        x if x == CjobCommandPause as i32 => Some(model::CJobCommand::Pause),
        x if x == CjobCommandResume as i32 => Some(model::CJobCommand::Resume),
        x if x == CjobCommandAbort as i32 => Some(model::CJobCommand::Abort),
        x if x == CjobCommandCancel as i32 => Some(model::CJobCommand::Cancel),
        x if x == CjobCommandStop as i32 => Some(model::CJobCommand::Stop),
        _ => None,
    }
}

pub fn cjob_command_to_proto(cmd: model::CJobCommand) -> i32 {
    use pb::e94::CJobCommand::*;
    match cmd {
        model::CJobCommand::Start => CjobCommandStart as i32,
        model::CJobCommand::Pause => CjobCommandPause as i32,
        model::CJobCommand::Resume => CjobCommandResume as i32,
        model::CJobCommand::Abort => CjobCommandAbort as i32,
        model::CJobCommand::Cancel => CjobCommandCancel as i32,
        model::CJobCommand::Stop => CjobCommandStop as i32,
    }
}

pub fn material_destination_from_proto(dest: i32) -> model::MaterialDestination {
    use pb::e94::MaterialDestination::*;
    match dest {
        x if x == SourceCarrier as i32 => model::MaterialDestination::SourceCarrier,
        x if x == DestinationCarrier as i32 => model::MaterialDestination::DestinationCarrier,
        x if x == Sorting as i32 => model::MaterialDestination::Sorting,
        _ => model::MaterialDestination::Unspecified,
    }
}

pub fn material_destination_to_proto(dest: model::MaterialDestination) -> i32 {
    use pb::e94::MaterialDestination::*;
    match dest {
        model::MaterialDestination::Unspecified => Unspecified as i32,
        model::MaterialDestination::SourceCarrier => SourceCarrier as i32,
        model::MaterialDestination::DestinationCarrier => DestinationCarrier as i32,
        model::MaterialDestination::Sorting => Sorting as i32,
    }
}

pub fn control_job_from_proto(job: pb::e94::ControlJob) -> Option<model::ControlJob> {
    Some(model::ControlJob {
        cjob_id: job.cjob_id,
        state: cjob_state_from_proto(job.state)?,
        pjob_ids: job.pjob_ids,
        destination: material_destination_from_proto(job.destination),
        destination_carrier_id: job.destination_carrier_id,
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
    })
}

pub fn control_job_to_proto(job: model::ControlJob) -> pb::e94::ControlJob {
    pb::e94::ControlJob {
        cjob_id: job.cjob_id,
        state: cjob_state_to_proto(job.state),
        pjob_ids: job.pjob_ids,
        destination: material_destination_to_proto(job.destination),
        destination_carrier_id: job.destination_carrier_id,
        created_at: job.created_at.map(|t| {
            let dur = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
            ::prost_types::Timestamp {
                seconds: dur.as_secs() as i64,
                nanos: dur.subsec_nanos() as i32,
            }
        }),
        started_at: job.started_at.map(|t| {
            let dur = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
            ::prost_types::Timestamp {
                seconds: dur.as_secs() as i64,
                nanos: dur.subsec_nanos() as i32,
            }
        }),
        completed_at: job.completed_at.map(|t| {
            let dur = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
            ::prost_types::Timestamp {
                seconds: dur.as_secs() as i64,
                nanos: dur.subsec_nanos() as i32,
            }
        }),
    }
}
