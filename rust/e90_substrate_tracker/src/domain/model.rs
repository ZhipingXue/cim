use std::time::SystemTime;

// ─────────────────────────────────────────────
// Domain types — pure Rust, no proto dependency
// ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstrateId {
    pub carrier_id: String,
    pub slot_number: i32,
    pub substrate_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstrateLocation {
    pub substrate: SubstrateId,
    pub location_id: String,
    pub location_type: String,
    pub timestamp: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstrateHistory {
    pub substrate: SubstrateId,
    pub locations: Vec<SubstrateLocation>,
}
