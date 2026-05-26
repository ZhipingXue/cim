use std::time::SystemTime;

use crate::domain::model;
use crate::pb;

pub fn substrate_id_from_proto(s: pb::common::SubstrateId) -> model::SubstrateId {
    model::SubstrateId {
        carrier_id: s.carrier_id,
        slot_number: s.slot_number,
        substrate_id: s.substrate_id,
    }
}

pub fn substrate_id_to_proto(s: model::SubstrateId) -> pb::common::SubstrateId {
    pb::common::SubstrateId {
        carrier_id: s.carrier_id,
        slot_number: s.slot_number,
        substrate_id: s.substrate_id,
    }
}

pub fn substrate_location_from_proto(loc: pb::e90::SubstrateLocation) -> model::SubstrateLocation {
    model::SubstrateLocation {
        substrate: loc.substrate.map(substrate_id_from_proto).unwrap_or(model::SubstrateId {
            carrier_id: String::new(),
            slot_number: 0,
            substrate_id: String::new(),
        }),
        location_id: loc.location_id,
        location_type: loc.location_type,
        timestamp: loc.timestamp.and_then(|t| {
            SystemTime::UNIX_EPOCH
                .checked_add(std::time::Duration::from_secs(t.seconds as u64))
                .and_then(|s| s.checked_add(std::time::Duration::from_nanos(t.nanos as u64)))
        }),
    }
}

pub fn substrate_location_to_proto(loc: model::SubstrateLocation) -> pb::e90::SubstrateLocation {
    pb::e90::SubstrateLocation {
        substrate: Some(substrate_id_to_proto(loc.substrate)),
        location_id: loc.location_id,
        location_type: loc.location_type,
        timestamp: loc.timestamp.map(|t| {
            let dur = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
            ::prost_types::Timestamp {
                seconds: dur.as_secs() as i64,
                nanos: dur.subsec_nanos() as i32,
            }
        }),
    }
}

pub fn substrate_history_from_proto(h: pb::e90::SubstrateHistory) -> model::SubstrateHistory {
    model::SubstrateHistory {
        substrate: h.substrate.map(substrate_id_from_proto).unwrap_or(model::SubstrateId {
            carrier_id: String::new(),
            slot_number: 0,
            substrate_id: String::new(),
        }),
        locations: h.locations.into_iter().map(substrate_location_from_proto).collect(),
    }
}

pub fn substrate_history_to_proto(h: model::SubstrateHistory) -> pb::e90::SubstrateHistory {
    pb::e90::SubstrateHistory {
        substrate: Some(substrate_id_to_proto(h.substrate)),
        locations: h.locations.into_iter().map(substrate_location_to_proto).collect(),
    }
}
