use async_trait::async_trait;
use tonic::{Request, Response, Status};

use crate::domain::mapper;
use crate::pb;
use crate::pb::e87::carrier_management_service_server::CarrierManagementService;
use crate::pb::e87::{
    CarrierRequest, CarrierResponse, LoadPortRequest, LoadPortResponse,
    SetAccessModeRequest, SlotMapVerificationRequest,
};
use crate::store::CarrierStore;
use infra::logging as infra_logging;

use semi_common::error::{CimError, CimResult};
use semi_common::traits::SemiService;

#[derive(Debug, Clone)]
pub struct CarrierService {
    store: CarrierStore,
}

impl Default for CarrierService {
    fn default() -> Self {
        Self::new()
    }
}

impl CarrierService {
    pub fn new() -> Self {
        Self {
            store: CarrierStore::new(),
        }
    }
}

#[async_trait]
impl SemiService for CarrierService {
    fn service_type(&self) -> &'static str {
        "e87_carrier_manager"
    }

    async fn initialize(&self) -> CimResult<()> {
        infra_logging::log_svc_lifecycle(infra_logging::event_names::SVC_INITIALIZE, "e87_carrier_manager");
        Ok(())
    }

    async fn shutdown(&self) -> CimResult<()> {
        infra_logging::log_svc_lifecycle(infra_logging::event_names::SVC_SHUTDOWN, "e87_carrier_manager");
        Ok(())
    }

    async fn health(&self) -> CimResult<()> {
        Ok(())
    }
}

#[tonic::async_trait]
impl CarrierManagementService for CarrierService {
    async fn get_load_port(
        &self,
        request: Request<LoadPortRequest>,
    ) -> Result<Response<LoadPortResponse>, Status> {
        let req = request.into_inner();
        infra_logging::log_svc_request("get_load_port", Some(&req.loadport_id));
        match self.store.get_loadport(&req.loadport_id).await {
            Some(lp) => {
                infra_logging::log_svc_response("get_load_port", Some(&req.loadport_id), true);
                Ok(Response::new(LoadPortResponse {
                    status: Some(pb::common::StatusCode {
                        code: 0,
                        message: "OK".to_string(),
                    }),
                    loadport: Some(mapper::loadport_to_proto(lp)),
                }))
            }
            None => {
                let msg = format!("LoadPort {} not found", req.loadport_id);
                infra_logging::log_svc_error("get_load_port", Some(&req.loadport_id), &msg);
                Err(Status::not_found(msg))
            }
        }
    }

    async fn get_carrier(
        &self,
        request: Request<CarrierRequest>,
    ) -> Result<Response<CarrierResponse>, Status> {
        let req = request.into_inner();
        infra_logging::log_svc_request("get_carrier", Some(&req.carrier_id));
        match self.store.get_carrier(&req.carrier_id).await {
            Some(c) => {
                infra_logging::log_svc_response("get_carrier", Some(&req.carrier_id), true);
                Ok(Response::new(CarrierResponse {
                    status: Some(pb::common::StatusCode {
                        code: 0,
                        message: "OK".to_string(),
                    }),
                    carrier: Some(mapper::carrier_to_proto(c)),
                }))
            }
            None => {
                let msg = format!("Carrier {} not found", req.carrier_id);
                infra_logging::log_svc_error("get_carrier", Some(&req.carrier_id), &msg);
                Err(Status::not_found(msg))
            }
        }
    }

    async fn set_access_mode(
        &self,
        request: Request<SetAccessModeRequest>,
    ) -> Result<Response<LoadPortResponse>, Status> {
        let req = request.into_inner();
        infra_logging::log_svc_request("set_access_mode", Some(&req.loadport_id));
        let mode = mapper::access_mode_from_proto(req.mode);
        match self.store.set_access_mode(&req.loadport_id, mode).await {
            Ok(lp) => {
                infra_logging::log_svc_response("set_access_mode", Some(&req.loadport_id), true);
                Ok(Response::new(LoadPortResponse {
                    status: Some(pb::common::StatusCode {
                        code: 0,
                        message: "OK".to_string(),
                    }),
                    loadport: Some(mapper::loadport_to_proto(lp)),
                }))
            }
            Err(CimError::NotFound(e)) => {
                infra_logging::log_svc_error("set_access_mode", Some(&req.loadport_id), &e);
                Err(Status::not_found(e))
            }
            Err(e) => {
                infra_logging::log_svc_error("set_access_mode", Some(&req.loadport_id), &e.to_string());
                Err(Status::internal(e.to_string()))
            }
        }
    }

    async fn verify_slot_map(
        &self,
        request: Request<SlotMapVerificationRequest>,
    ) -> Result<Response<CarrierResponse>, Status> {
        let req = request.into_inner();
        infra_logging::log_svc_request("verify_slot_map", Some(&req.carrier_id));
        match self.store.verify_slot_map(&req.carrier_id, &req.expected_slot_map).await {
            Ok(c) => {
                infra_logging::log_svc_response("verify_slot_map", Some(&req.carrier_id), true);
                Ok(Response::new(CarrierResponse {
                    status: Some(pb::common::StatusCode {
                        code: 0,
                        message: "OK".to_string(),
                    }),
                    carrier: Some(mapper::carrier_to_proto(c)),
                }))
            }
            Err(CimError::NotFound(e)) => {
                infra_logging::log_svc_error("verify_slot_map", Some(&req.carrier_id), &e);
                Err(Status::not_found(e))
            }
            Err(e) => {
                infra_logging::log_svc_error("verify_slot_map", Some(&req.carrier_id), &e.to_string());
                Err(Status::internal(e.to_string()))
            }
        }
    }

    type StreamCarrierEventsStream =
        tokio_stream::wrappers::ReceiverStream<Result<pb::common::Event, Status>>;

    async fn stream_carrier_events(
        &self,
        _request: Request<pb::common::Empty>,
    ) -> Result<Response<Self::StreamCarrierEventsStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        tokio::spawn(async move {
            let _ = tx.send(Ok(pb::common::Event {
                event_id: "evt-1".to_string(),
                event_name: "CarrierEvent".to_string(),
                timestamp: Some(std::time::SystemTime::now().into()),
                source: Some(pb::common::ModuleId {
                    equipment_id: "eq-001".to_string(),
                    module_id: "e87-cms".to_string(),
                    module_type: "service".to_string(),
                }),
                parameters: std::collections::HashMap::new(),
            })).await;
        });
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}
