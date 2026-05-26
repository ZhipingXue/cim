use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::domain::model::{SubstrateHistory, SubstrateId, SubstrateLocation};
use infra::logging as infra_logging;

#[derive(Debug, Clone)]
pub struct TrackingStore {
    histories: Arc<RwLock<HashMap<String, SubstrateHistory>>>,
}

impl Default for TrackingStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackingStore {
    pub fn new() -> Self {
        Self {
            histories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn track(
        &self,
        substrate_id: String,
        location: SubstrateLocation,
    ) {
        let mut histories = self.histories.write().await;
        let history = histories.entry(substrate_id.clone()).or_insert_with(|| SubstrateHistory {
            substrate: location.substrate.clone(),
            locations: vec![],
        });
        history.locations.push(location);
        infra_logging::log_store_create(&substrate_id, "substrate", "tracked");
    }

    pub async fn get_history(
        &self,
        substrate_id: &str,
    ) -> Option<SubstrateHistory> {
        let histories = self.histories.read().await;
        let h = histories.get(substrate_id).cloned();
        infra_logging::log_store_get(substrate_id, "substrate", h.is_some());
        h
    }

    pub async fn get_batch_history(
        &self,
        substrate_ids: &[ String],
    ) -> Vec<SubstrateHistory> {
        let histories = self.histories.read().await;
        substrate_ids
            .iter()
            .filter_map(|id| histories.get(id).cloned())
            .collect()
    }
}
