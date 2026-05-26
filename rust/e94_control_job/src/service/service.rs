use std::sync::Arc;
use std::time::SystemTime;

use async_trait::async_trait;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::domain::{mapper, model};
use crate::pb;
use crate::pb::e94::control_job_service_server::ControlJobService;
use crate::pb::e94::{
    CreateCJobRequest, CreateCJobResponse, GetCJobRequest, GetCJobResponse,
    ListCJobsRequest, ListCJobsResponse, CJobCommandRequest, CJobCommandResponse,
};
use crate::state_machine::executor::NoOpCJobExecutor;
use crate::store::CJobStore;
use infra::logging as infra_logging;

use semi_common::error::{CimError, CimResult};
use semi_common::traits::SemiService;

#[derive(Debug, Clone)]
pub struct CJobService {
    store: CJobStore<NoOpCJobExecutor>,
}

impl Default for CJobService {
    fn default() -> Self {
        Self::new()
    }
}

impl CJobService {
    pub fn new() -> Self {
        Self {
            store: CJobStore::new(Arc::new(NoOpCJobExecutor)),
        }
    }
}

#[async_trait]
impl SemiService for CJobService {
    fn service_type(&self) -> &'static str {
        "e94_control_job"
    }

    async fn initialize(&self) -> CimResult<()> {
        infra_logging::log_svc_lifecycle(infra_logging::event_names::SVC_INITIALIZE, "e94_control_job");
        Ok(())
    }

    async fn shutdown(&self) -> CimResult<()> {
        infra_logging::log_svc_lifecycle(infra_logging::event_names::SVC_SHUTDOWN, "e94_control_job");
        Ok(())
    }

    async fn health(&self) -> CimResult<()> {
        Ok(())
    }
}

#[tonic::async_trait]
impl ControlJobService for CJobService {
    async fn create_c_job(
        &self,
        request: Request<CreateCJobRequest>,
    ) -> Result<Response<CreateCJobResponse>, Status> {
        let req = request.into_inner();
        infra_logging::log_svc_request("create_c_job", Some(&req.cjob_id));

        let job = model::ControlJob {
            cjob_id: req.cjob_id.clone(),
            state: model::CJobState::Queued,
            pjob_ids: req.pjob_ids,
            destination: mapper::material_destination_from_proto(req.destination),
            destination_carrier_id: req.destination_carrier_id,
            created_at: Some(SystemTime::now()),
            started_at: None,
            completed_at: None,
        };

        match self.store.create(job).await {
            Ok(job) => {
                infra_logging::log_svc_response("create_c_job", Some(&req.cjob_id), true);
                Ok(Response::new(CreateCJobResponse {
                    status: Some(pb::common::StatusCode {
                        code: 0,
                        message: "OK".to_string(),
                    }),
                    cjob: Some(mapper::control_job_to_proto(job)),
                }))
            }
            Err(CimError::AlreadyExists(e)) => {
                infra_logging::log_svc_error("create_c_job", Some(&req.cjob_id), &e);
                Err(Status::already_exists(e))
            }
            Err(e) => {
                infra_logging::log_svc_error("create_c_job", Some(&req.cjob_id), &e.to_string());
                Err(Status::internal(e.to_string()))
            }
        }
    }

    async fn get_c_job(
        &self,
        request: Request<GetCJobRequest>,
    ) -> Result<Response<GetCJobResponse>, Status> {
        let req = request.into_inner();
        infra_logging::log_svc_request("get_c_job", Some(&req.cjob_id));

        match self.store.get(&req.cjob_id).await {
            Some(job) => {
                infra_logging::log_svc_response("get_c_job", Some(&req.cjob_id), true);
                Ok(Response::new(GetCJobResponse {
                    status: Some(pb::common::StatusCode {
                        code: 0,
                        message: "OK".to_string(),
                    }),
                    cjob: Some(mapper::control_job_to_proto(job)),
                }))
            }
            None => {
                let msg = format!("CJob {} not found", req.cjob_id);
                infra_logging::log_svc_error("get_c_job", Some(&req.cjob_id), &msg);
                Err(Status::not_found(msg))
            }
        }
    }

    async fn list_c_jobs(
        &self,
        request: Request<ListCJobsRequest>,
    ) -> Result<Response<ListCJobsResponse>, Status> {
        infra_logging::log_svc_request("list_c_jobs", None);
        let req = request.into_inner();
        let filter_state = mapper::cjob_state_from_proto(req.filter_state);
        let jobs = self.store.list(filter_state).await;
        infra_logging::log_svc_response("list_c_jobs", None, true);
        Ok(Response::new(ListCJobsResponse {
            cjobs: jobs.into_iter().map(mapper::control_job_to_proto).collect(),
        }))
    }

    async fn execute_command(
        &self,
        request: Request<CJobCommandRequest>,
    ) -> Result<Response<CJobCommandResponse>, Status> {
        let req = request.into_inner();
        let cmd = mapper::cjob_command_from_proto(req.command)
            .ok_or_else(|| {
                let msg = format!("Invalid command: {}", req.command);
                infra_logging::log_svc_error("execute_command", Some(&req.cjob_id), &msg);
                Status::invalid_argument(msg)
            })?;

        infra_logging::log_svc_request("execute_command", Some(&req.cjob_id));

        match self.store.execute_command(&req.cjob_id, cmd).await {
            Ok(job) => {
                infra_logging::log_svc_response("execute_command", Some(&req.cjob_id), true);
                Ok(Response::new(CJobCommandResponse {
                    status: Some(pb::common::StatusCode {
                        code: 0,
                        message: "OK".to_string(),
                    }),
                    cjob: Some(mapper::control_job_to_proto(job)),
                }))
            }
            Err(CimError::NotFound(e)) => {
                infra_logging::log_svc_error("execute_command", Some(&req.cjob_id), &e);
                Err(Status::not_found(e))
            }
            Err(CimError::InvalidArgument(e)) => {
                infra_logging::log_svc_error("execute_command", Some(&req.cjob_id), &e);
                Err(Status::invalid_argument(e))
            }
            Err(e) => {
                infra_logging::log_svc_error("execute_command", Some(&req.cjob_id), &e.to_string());
                Err(Status::internal(e.to_string()))
            }
        }
    }

    type StreamCJobEventsStream =
        tokio_stream::wrappers::ReceiverStream<Result<pb::common::Event, Status>>;

    async fn stream_c_job_events(
        &self,
        _request: Request<pb::common::Empty>,
    ) -> Result<Response<Self::StreamCJobEventsStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        tokio::spawn(async move {
            let _ = tx.send(Ok(pb::common::Event {
                event_id: "evt-1".to_string(),
                event_name: "CJobCreated".to_string(),
                timestamp: Some(SystemTime::now().into()),
                source: Some(pb::common::ModuleId {
                    equipment_id: "eq-001".to_string(),
                    module_id: "e94-cjob".to_string(),
                    module_type: "service".to_string(),
                }),
                parameters: std::collections::HashMap::new(),
            })).await;
        });
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}
