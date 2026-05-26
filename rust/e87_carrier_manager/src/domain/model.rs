use std::time::SystemTime;

// ─────────────────────────────────────────────
// Domain types — pure Rust, no proto dependency
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CarrierState {
    Unspecified,
    WaitingForCarrier,
    CarrierIdRead,
    Verifying,
    VerifyCompleted,
    VerifyFailed,
    WaitingForHost,
    ReadyToLoad,
    ReadyToUnload,
    TransferBlocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadPortTransferState {
    Unspecified,
    OutOfService,
    InService,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccessMode {
    #[default]
    Unspecified,
    Manual,
    Auto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadPort {
    pub loadport_id: String,
    pub transfer_state: LoadPortTransferState,
    pub access_mode: AccessMode,
    pub associated_carrier_id: String,
    pub reserved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Carrier {
    pub carrier_id: String,
    pub state: CarrierState,
    pub loadport_id: String,
    pub slot_map: Vec<String>,
    pub arrived_at: Option<SystemTime>,
}
