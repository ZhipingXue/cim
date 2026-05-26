use std::time::SystemTime;

use crate::domain::model;
use crate::pb;

pub fn carrier_state_from_proto(state: i32) -> Option<model::CarrierState> {
    use pb::e87::CarrierState::*;
    match state {
        x if x == CarrierWaitingForCarrier as i32 => Some(model::CarrierState::WaitingForCarrier),
        x if x == CarrierCarrierIdRead as i32 => Some(model::CarrierState::CarrierIdRead),
        x if x == CarrierVerifying as i32 => Some(model::CarrierState::Verifying),
        x if x == CarrierVerifyCompleted as i32 => Some(model::CarrierState::VerifyCompleted),
        x if x == CarrierVerifyFailed as i32 => Some(model::CarrierState::VerifyFailed),
        x if x == CarrierWaitingForHost as i32 => Some(model::CarrierState::WaitingForHost),
        x if x == CarrierReadyToLoad as i32 => Some(model::CarrierState::ReadyToLoad),
        x if x == CarrierReadyToUnload as i32 => Some(model::CarrierState::ReadyToUnload),
        x if x == CarrierTransferBlocked as i32 => Some(model::CarrierState::TransferBlocked),
        _ => Some(model::CarrierState::Unspecified),
    }
}

pub fn carrier_state_to_proto(state: model::CarrierState) -> i32 {
    use pb::e87::CarrierState::*;
    match state {
        model::CarrierState::Unspecified => CarrierUnspecified as i32,
        model::CarrierState::WaitingForCarrier => CarrierWaitingForCarrier as i32,
        model::CarrierState::CarrierIdRead => CarrierCarrierIdRead as i32,
        model::CarrierState::Verifying => CarrierVerifying as i32,
        model::CarrierState::VerifyCompleted => CarrierVerifyCompleted as i32,
        model::CarrierState::VerifyFailed => CarrierVerifyFailed as i32,
        model::CarrierState::WaitingForHost => CarrierWaitingForHost as i32,
        model::CarrierState::ReadyToLoad => CarrierReadyToLoad as i32,
        model::CarrierState::ReadyToUnload => CarrierReadyToUnload as i32,
        model::CarrierState::TransferBlocked => CarrierTransferBlocked as i32,
    }
}

pub fn loadport_transfer_state_from_proto(state: i32) -> model::LoadPortTransferState {
    use pb::e87::LoadPortTransferState::*;
    match state {
        x if x == LpTransferOutOfService as i32 => model::LoadPortTransferState::OutOfService,
        x if x == LpTransferInService as i32 => model::LoadPortTransferState::InService,
        x if x == LpTransferBlocked as i32 => model::LoadPortTransferState::Blocked,
        _ => model::LoadPortTransferState::Unspecified,
    }
}

pub fn loadport_transfer_state_to_proto(state: model::LoadPortTransferState) -> i32 {
    use pb::e87::LoadPortTransferState::*;
    match state {
        model::LoadPortTransferState::Unspecified => LpTransferUnspecified as i32,
        model::LoadPortTransferState::OutOfService => LpTransferOutOfService as i32,
        model::LoadPortTransferState::InService => LpTransferInService as i32,
        model::LoadPortTransferState::Blocked => LpTransferBlocked as i32,
    }
}

pub fn access_mode_from_proto(mode: i32) -> model::AccessMode {
    use pb::e87::AccessMode::*;
    match mode {
        x if x == AccessManual as i32 => model::AccessMode::Manual,
        x if x == AccessAuto as i32 => model::AccessMode::Auto,
        _ => model::AccessMode::Unspecified,
    }
}

pub fn access_mode_to_proto(mode: model::AccessMode) -> i32 {
    use pb::e87::AccessMode::*;
    match mode {
        model::AccessMode::Unspecified => AccessUnspecified as i32,
        model::AccessMode::Manual => AccessManual as i32,
        model::AccessMode::Auto => AccessAuto as i32,
    }
}

pub fn loadport_from_proto(lp: pb::e87::LoadPort) -> model::LoadPort {
    model::LoadPort {
        loadport_id: lp.loadport_id,
        transfer_state: loadport_transfer_state_from_proto(lp.transfer_state),
        access_mode: access_mode_from_proto(lp.access_mode),
        associated_carrier_id: lp.associated_carrier_id,
        reserved: lp.reserved,
    }
}

pub fn loadport_to_proto(lp: model::LoadPort) -> pb::e87::LoadPort {
    pb::e87::LoadPort {
        loadport_id: lp.loadport_id,
        transfer_state: loadport_transfer_state_to_proto(lp.transfer_state),
        access_mode: access_mode_to_proto(lp.access_mode),
        associated_carrier_id: lp.associated_carrier_id,
        reserved: lp.reserved,
    }
}

pub fn carrier_from_proto(c: pb::e87::Carrier) -> Option<model::Carrier> {
    Some(model::Carrier {
        carrier_id: c.carrier_id,
        state: carrier_state_from_proto(c.state)?,
        loadport_id: c.loadport_id,
        slot_map: c.slot_map,
        arrived_at: c.arrived_at.and_then(|t| {
            SystemTime::UNIX_EPOCH
                .checked_add(std::time::Duration::from_secs(t.seconds as u64))
                .and_then(|s| s.checked_add(std::time::Duration::from_nanos(t.nanos as u64)))
        }),
    })
}

pub fn carrier_to_proto(c: model::Carrier) -> pb::e87::Carrier {
    pb::e87::Carrier {
        carrier_id: c.carrier_id,
        state: carrier_state_to_proto(c.state),
        loadport_id: c.loadport_id,
        slot_map: c.slot_map,
        arrived_at: c.arrived_at.map(|t| {
            let dur = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
            ::prost_types::Timestamp {
                seconds: dur.as_secs() as i64,
                nanos: dur.subsec_nanos() as i32,
            }
        }),
    }
}
