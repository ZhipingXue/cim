use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleId {
    pub equipment_id: String,
    pub module_id: String,
    pub module_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstrateId {
    pub carrier_id: String,
    pub slot_number: i32,
    pub substrate_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarrierId {
    pub carrier_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alarm {
    pub alarm_id: i32,
    pub alarm_code: String,
    pub description: String,
    pub severity: AlarmSeverity,
    pub timestamp: DateTime<Utc>,
    pub source: ModuleId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlarmSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: String,
    pub event_name: String,
    pub timestamp: DateTime<Utc>,
    pub source: ModuleId,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub service_id: String,
    pub service_type: ServiceType,
    pub host: String,
    pub port: i32,
    pub module_id: Option<String>,
    pub registered_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub healthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    E40ProcessJob,
    E94ControlJob,
    E87CarrierManager,
    E90SubstrateTracker,
    E125Metadata,
    E134DataCollection,
    ControlApp,
}

impl ServiceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceType::E40ProcessJob => "e40_process_job",
            ServiceType::E94ControlJob => "e94_control_job",
            ServiceType::E87CarrierManager => "e87_carrier_manager",
            ServiceType::E90SubstrateTracker => "e90_substrate_tracker",
            ServiceType::E125Metadata => "e125_metadata_manager",
            ServiceType::E134DataCollection => "e134_data_collection",
            ServiceType::ControlApp => "control_app",
        }
    }
}
