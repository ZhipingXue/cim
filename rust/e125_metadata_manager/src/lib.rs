pub mod pb {
    pub mod common {
        tonic::include_proto!("cim.common");
    }
    pub mod e125 {
        tonic::include_proto!("cim.e125");
    }
}

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use pb::e125::metadata_service_server::{MetadataService, MetadataServiceServer};
use pb::e125::{
    EquipmentCapability, EquipmentMetadata, GetMetadataRequest, GetMetadataResponse,
    ListModulesRequest, ListModulesResponse, ModuleInfo,
};

#[derive(Debug, Clone)]
pub struct MetadataStore {
    metadata: Arc<RwLock<HashMap<String, EquipmentMetadata>>>,
}

impl MetadataStore {
    pub fn new() -> Self {
        let mut metadata = HashMap::new();
        // Insert placeholder metadata
        metadata.insert(
            "eq-1".to_string(),
            EquipmentMetadata {
                equipment_id: "eq-1".to_string(),
                equipment_model: "CIM-300mm-01".to_string(),
                vendor: "CIM Corp".to_string(),
                software_version: "1.0.0".to_string(),
                capabilities: vec![
                    EquipmentCapability {
                        standard_name: "E40".to_string(),
                        version: "1.0".to_string(),
                        features: vec!["process_job".to_string()],
                    },
                    EquipmentCapability {
                        standard_name: "E94".to_string(),
                        version: "1.0".to_string(),
                        features: vec!["control_job".to_string()],
                    },
                    EquipmentCapability {
                        standard_name: "E87".to_string(),
                        version: "1.0".to_string(),
                        features: vec!["carrier_management".to_string()],
                    },
                    EquipmentCapability {
                        standard_name: "E90".to_string(),
                        version: "1.0".to_string(),
                        features: vec!["substrate_tracking".to_string()],
                    },
                ],
                modules: vec![
                    ModuleInfo {
                        module_id: Some(pb::common::ModuleId {
                            equipment_id: "eq-1".to_string(),
                            module_id: "chamber-01".to_string(),
                            module_type: "chamber".to_string(),
                        }),
                        description: "Process Chamber 1".to_string(),
                        supported_operations: vec!["process".to_string()],
                    },
                    ModuleInfo {
                        module_id: Some(pb::common::ModuleId {
                            equipment_id: "eq-1".to_string(),
                            module_id: "robot-01".to_string(),
                            module_type: "robot".to_string(),
                        }),
                        description: "Transfer Robot".to_string(),
                        supported_operations: vec!["pick".to_string(), "place".to_string()],
                    },
                ],
            },
        );

        Self {
            metadata: Arc::new(RwLock::new(metadata)),
        }
    }

    pub async fn get(&self, equipment_id: &str) -> Option<EquipmentMetadata> {
        let metadata = self.metadata.read().await;
        metadata.get(equipment_id).cloned()
    }

    pub async fn list_modules(&self, equipment_id: &str) -> Vec<ModuleInfo> {
        let metadata = self.metadata.read().await;
        metadata
            .get(equipment_id)
            .map(|m| m.modules.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct MetadataServiceImpl {
    store: MetadataStore,
}

impl MetadataServiceImpl {
    pub fn new() -> Self {
        Self {
            store: MetadataStore::new(),
        }
    }
}

#[tonic::async_trait]
impl MetadataService for MetadataServiceImpl {
    async fn get_metadata(
        &self,
        request: Request<GetMetadataRequest>,
    ) -> Result<Response<GetMetadataResponse>, Status> {
        let req = request.into_inner();
        match self.store.get(&req.equipment_id).await {
            Some(meta) => Ok(Response::new(GetMetadataResponse {
                status: Some(pb::common::StatusCode {
                    code: 0,
                    message: "OK".to_string(),
                }),
                metadata: Some(meta),
            })),
            None => Err(Status::not_found(format!(
                "Metadata for equipment {} not found",
                req.equipment_id
            ))),
        }
    }

    async fn list_modules(
        &self,
        request: Request<ListModulesRequest>,
    ) -> Result<Response<ListModulesResponse>, Status> {
        let req = request.into_inner();
        let modules = self.store.list_modules(&req.equipment_id).await;
        Ok(Response::new(ListModulesResponse {
            status: Some(pb::common::StatusCode {
                code: 0,
                message: "OK".to_string(),
            }),
            modules,
        }))
    }
}
