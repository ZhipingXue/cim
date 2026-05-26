use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::RwLock;
use tracing::info;

use crate::domain::model::{Carrier, CarrierState, LoadPort, AccessMode};
use infra::logging as infra_logging;
use semi_common::error::{CimError, CimResult};

#[derive(Debug, Clone)]
pub struct CarrierStore {
    loadports: Arc<RwLock<HashMap<String, LoadPort>>>,
    carriers: Arc<RwLock<HashMap<String, Carrier>>>,
}

impl Default for CarrierStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CarrierStore {
    pub fn new() -> Self {
        Self {
            loadports: Arc::new(RwLock::new(HashMap::new())),
            carriers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_loadport(&self,
        loadport_id: &str,
    ) -> Option<LoadPort> {
        let loadports = self.loadports.read().await;
        let lp = loadports.get(loadport_id).cloned();
        infra_logging::log_store_get(loadport_id, "loadport", lp.is_some());
        lp
    }

    pub async fn get_carrier(&self,
        carrier_id: &str,
    ) -> Option<Carrier> {
        let carriers = self.carriers.read().await;
        let c = carriers.get(carrier_id).cloned();
        infra_logging::log_store_get(carrier_id, "carrier", c.is_some());
        c
    }

    pub async fn create_loadport(&self,
        loadport: LoadPort,
    ) -> CimResult<LoadPort> {
        let mut loadports = self.loadports.write().await;
        if loadports.contains_key(&loadport.loadport_id) {
            infra_logging::log_store_create_duplicate(&loadport.loadport_id, "loadport");
            return Err(CimError::AlreadyExists(format!(
                "LoadPort {} already exists",
                loadport.loadport_id
            )));
        }
        loadports.insert(loadport.loadport_id.clone(), loadport.clone());
        infra_logging::log_store_create(&loadport.loadport_id, "loadport", &format!("{:?}", loadport.transfer_state));
        Ok(loadport)
    }

    pub async fn create_carrier(
        &self,
        carrier: Carrier,
    ) -> CimResult<Carrier> {
        let mut carriers = self.carriers.write().await;
        if carriers.contains_key(&carrier.carrier_id) {
            infra_logging::log_store_create_duplicate(&carrier.carrier_id, "carrier");
            return Err(CimError::AlreadyExists(format!(
                "Carrier {} already exists",
                carrier.carrier_id
            )));
        }
        carriers.insert(carrier.carrier_id.clone(), carrier.clone());
        infra_logging::log_store_create(&carrier.carrier_id, "carrier", &format!("{:?}", carrier.state));
        Ok(carrier)
    }

    pub async fn set_access_mode(
        &self,
        loadport_id: &str,
        mode: AccessMode,
    ) -> CimResult<LoadPort> {
        let mut loadports = self.loadports.write().await;
        let lp = loadports
            .get_mut(loadport_id)
            .ok_or_else(|| CimError::NotFound(format!("LoadPort {} not found", loadport_id)))?;
        lp.access_mode = mode;
        infra_logging::log_store_update(loadport_id, "loadport", "access_mode");
        infra_logging::log_store_update(loadport_id, "loadport", "access_mode");
        Ok(lp.clone())
    }

    pub async fn verify_slot_map(
        &self,
        carrier_id: &str,
        expected: &[ String],
    ) -> CimResult<Carrier> {
        let mut carriers = self.carriers.write().await;
        let carrier = carriers
            .get_mut(carrier_id)
            .ok_or_else(|| CimError::NotFound(format!("Carrier {} not found", carrier_id)))?;
        let matches = carrier.slot_map == expected;
        carrier.state = if matches {
            CarrierState::VerifyCompleted
        } else {
            CarrierState::VerifyFailed
        };
        infra_logging::log_store_update(carrier_id, "carrier", "slot_map_verification");
        Ok(carrier.clone())
    }

    pub async fn list_carriers(&self,
        filter_state: Option<CarrierState>,
    ) -> Vec<Carrier> {
        let carriers = self.carriers.read().await;
        carriers
            .values()
            .filter(|c| filter_state.map_or(true, |s| c.state == s))
            .cloned()
            .collect()
    }

    pub async fn list_loadports(&self) -> Vec<LoadPort> {
        let loadports = self.loadports.read().await;
        loadports.values().cloned().collect()
    }
}
