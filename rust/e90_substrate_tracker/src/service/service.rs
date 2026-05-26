use std::time::SystemTime;

use async_trait::async_trait;
use tonic::{Request, Response, Status};

use crate::domain::{mapper, model};
use crate::pb;
use crate::pb::e90::substrate_tracking_service_server::SubstrateTrackingService;
use crate::pb::e90::{
    GetSubstrateHistoryRequest, GetSubstrateHistoryResponse,
    TrackSubstrateRequest, TrackSubstrateResponse,
};
use crate::store::TrackingStore;
use infra::logging as infra_logging;

use semi_common::error::{CimError, CimResult};
use semi_common::traits::SemiService;

#[derive(Debug, Clone)]
pub struct TrackingService {
    store: TrackingStore,
}

impl Default for TrackingService {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackingService {
    pub fn new() -> Self {
        Self {
            store: TrackingStore::new(),
        }
    }
}

#[async_trait]
impl SemiService for TrackingService {
    fn service_type(&self) -> &'static str {
        "e90_substrate_tracker"
    }

    async fn initialize(&self) -> CimResult<()> {
        infra_logging::log_svc_lifecycle(infra_logging::event_names::SVC_INITIALIZE, "e90_substrate_tracker");
        Ok(())
    }

    async fn shutdown(&self) -> CimResult<()> {
        infra_logging::log_svc_lifecycle(infra_logging::event_names::SVC_SHUTDOWN, "e90_substrate_tracker");
        Ok(())
    }

    async fn health(&self) -> CimResult<()> {
        Ok(())
    }
}

#[tonic::async_trait]
impl SubstrateTrackingService for TrackingService {
    async fn track_substrate(
        &self,
        request: Request<TrackSubstrateRequest>,
    ) -> Result<Response<TrackSubstrateResponse>, Status> {
        let req = request.into_inner();
        let substrate_id = req.substrate.as_ref()
            .map(|s| s.substrate_id.clone())
            .unwrap_or_default();
        
        infra_logging::log_svc_request("track_substrate", Some(&substrate_id));
        
        let location = model::SubstrateLocation {
            substrate: req.substrate.map(mapper::substrate_id_from_proto).unwrap_or(model::SubstrateId {
                carrier_id: String::new(),
                slot_number: 0,
                substrate_id: substrate_id.clone(),
            }),
            location_id: req.location_id,
            location_type: req.location_type,
            timestamp: Some(SystemTime::now()),
        };
        
        self.store.track(substrate_id.clone(), location.clone()).await;
        
        infra_logging::log_svc_response("track_substrate", Some(&substrate_id), true);
        Ok(Response::new(TrackSubstrateResponse {
            status: Some(pb::common::StatusCode {
                code: 0,
                message: "OK".to_string(),
            }),
            location: Some(mapper::substrate_location_to_proto(location)),
        }))
    }

    async fn get_substrate_history(
        &self,
        request: Request<GetSubstrateHistoryRequest>,
    ) -> Result<Response<GetSubstrateHistoryResponse>, Status> {
        let req = request.into_inner();
        let substrate_id = req.substrate.as_ref()
            .map(|s| s.substrate_id.clone())
            .unwrap_or_default();
        
        infra_logging::log_svc_request("get_substrate_history", Some(&substrate_id));
        
        match self.store.get_history(&substrate_id).await {
            Some(history) => {
                infra_logging::log_svc_response("get_substrate_history", Some(&substrate_id), true);
                Ok(Response::new(GetSubstrateHistoryResponse {
                    status: Some(pb::common::StatusCode {
                        code: 0,
                        message: "OK".to_string(),
                    }),
                    history: Some(mapper::substrate_history_to_proto(history)),
                }))
            }
            None => {
                let msg = format!("Substrate {} not found", substrate_id);
                infra_logging::log_svc_error("get_substrate_history", Some(&substrate_id), &msg);
                Err(Status::not_found(msg))
            }
        }
    }

    type GetBatchHistoryStream =
        tokio_stream::wrappers::ReceiverStream<Result<GetSubstrateHistoryResponse, Status>>;

    async fn get_batch_history(
        &self,
        request: Request<pb::e90::BatchRequest>,
    ) -> Result<Response<Self::GetBatchHistoryStream>, Status> {
        let req = request.into_inner();
        let substrate_ids: Vec<String> = req.substrates
            .into_iter()
            .map(|s| s.substrate_id)
            .collect();
        
        infra_logging::log_svc_request("get_batch_history", None);
        let histories = self.store.get_batch_history(&substrate_ids).await;
        infra_logging::log_svc_response("get_batch_history", None, true);
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        
        tokio::spawn(async move {
            for history in histories {
                let _ = tx.send(Ok(GetSubstrateHistoryResponse {
                    status: Some(pb::common::StatusCode {
                        code: 0,
                        message: "OK".to_string(),
                    }),
                    history: Some(mapper::substrate_history_to_proto(history)),
                })).await;
            }
        });
        
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    type StreamLocationEventsStream =
        tokio_stream::wrappers::ReceiverStream<Result<pb::common::Event, Status>>;

    async fn stream_location_events(
        &self,
        _request: Request<pb::common::Empty>,
    ) -> Result<Response<Self::StreamLocationEventsStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        tokio::spawn(async move {
            let _ = tx.send(Ok(pb::common::Event {
                event_id: "evt-1".to_string(),
                event_name: "LocationTracked".to_string(),
                timestamp: Some(SystemTime::now().into()),
                source: Some(pb::common::ModuleId {
                    equipment_id: "eq-001".to_string(),
                    module_id: "e90-sts".to_string(),
                    module_type: "service".to_string(),
                }),
                parameters: std::collections::HashMap::new(),
            })).await;
        });
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}
