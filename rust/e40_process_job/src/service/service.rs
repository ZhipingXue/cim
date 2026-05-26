use std::sync::Arc;
use std::time::SystemTime;

use async_trait::async_trait;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::domain::{mapper, model};
use crate::pb;
use crate::pb::e40::process_job_service_server::ProcessJobService;
use crate::pb::e40::{
    CreatePJobRequest, CreatePJobResponse, GetPJobRequest, GetPJobResponse,
    ListPJobsRequest, ListPJobsResponse, PJobCommandRequest, PJobCommandResponse,
};
use crate::state_machine::executor::NoOpPJobExecutor;
use infra::logging as infra_logging;
use crate::store::PJobStore;

use semi_common::error::{CimError, CimResult};
use semi_common::traits::SemiService;

#[derive(Debug, Clone)]
pub struct PJobService {
    store: PJobStore<NoOpPJobExecutor>,
}

impl Default for PJobService {
    fn default() -> Self {
        Self::new()
    }
}

impl PJobService {
    pub fn new() -> Self {
        Self {
            store: PJobStore::new(Arc::new(NoOpPJobExecutor)),
        }
    }
}

#[async_trait]
impl SemiService for PJobService {
    fn service_type(&self) -> &'static str {
        "e40_process_job"
    }

    async fn initialize(&self) -> CimResult<()> {
        infra_logging::log_svc_lifecycle(infra_logging::event_names::SVC_INITIALIZE, "e40_process_job");
        Ok(())
    }

    async fn shutdown(&self) -> CimResult<()> {
        infra_logging::log_svc_lifecycle(infra_logging::event_names::SVC_SHUTDOWN, "e40_process_job");
        Ok(())
    }

    async fn health(&self) -> CimResult<()> {
        Ok(())
    }
}

#[tonic::async_trait]
impl ProcessJobService for PJobService {
    async fn create_p_job(
        &self,
        request: Request<CreatePJobRequest>,
    ) -> Result<Response<CreatePJobResponse>, Status> {
        let req = request.into_inner();
        infra_logging::log_svc_request("create_p_job", Some(&req.pjob_id));

        let job = model::ProcessJob {
            pjob_id: req.pjob_id.clone(),
            state: model::PJobState::Queued,
            materials: req.materials.into_iter().map(mapper::material_from_proto).collect(),
            recipe: mapper::recipe_from_proto(req.recipe),
            cjob_id: req.cjob_id,
            created_at: Some(SystemTime::now()),
            started_at: None,
            completed_at: None,
            pr_process_start: req.pr_process_start,
            pr_recipe_method: mapper::recipe_method_from_proto(req.pr_recipe_method),
            recipe_variables: req
                .recipe_variables
                .into_iter()
                .map(mapper::recipe_variable_from_proto)
                .collect(),
        };

        match self.store.create(job).await {
            Ok(job) => {
                infra_logging::log_svc_response("create_p_job", Some(&req.pjob_id), true);
                Ok(Response::new(CreatePJobResponse {
                    status: Some(pb::common::StatusCode {
                        code: 0,
                        message: "OK".to_string(),
                    }),
                    pjob: Some(mapper::process_job_to_proto(job)),
                }))
            }
            Err(CimError::AlreadyExists(e)) => {
                infra_logging::log_svc_error("create_p_job", Some(&req.pjob_id), &e);
                Err(Status::already_exists(e))
            }
            Err(e) => {
                infra_logging::log_svc_error("create_p_job", Some(&req.pjob_id), &e.to_string());
                Err(Status::internal(e.to_string()))
            }
        }
    }

    async fn get_p_job(
        &self,
        request: Request<GetPJobRequest>,
    ) -> Result<Response<GetPJobResponse>, Status> {
        let req = request.into_inner();
        infra_logging::log_svc_request("get_p_job", Some(&req.pjob_id));

        match self.store.get(&req.pjob_id).await {
            Some(job) => {
                infra_logging::log_svc_response("get_p_job", Some(&req.pjob_id), true);
                Ok(Response::new(GetPJobResponse {
                    status: Some(pb::common::StatusCode {
                        code: 0,
                        message: "OK".to_string(),
                    }),
                    pjob: Some(mapper::process_job_to_proto(job)),
                }))
            }
            None => {
                let msg = format!("PJob {} not found", req.pjob_id);
                infra_logging::log_svc_error("get_p_job", Some(&req.pjob_id), &msg);
                Err(Status::not_found(msg))
            }
        }
    }

    async fn list_p_jobs(
        &self,
        request: Request<ListPJobsRequest>,
    ) -> Result<Response<ListPJobsResponse>, Status> {
        infra_logging::log_svc_request("list_p_jobs", None);
        let req = request.into_inner();
        let filter_state = mapper::pjob_state_from_proto(req.filter_state);
        let jobs = self.store.list(filter_state, &req.cjob_id).await;
        infra_logging::log_svc_response("list_p_jobs", None, true);
        Ok(Response::new(ListPJobsResponse {
            pjobs: jobs.into_iter().map(mapper::process_job_to_proto).collect(),
        }))
    }

    async fn execute_command(
        &self,
        request: Request<PJobCommandRequest>,
    ) -> Result<Response<PJobCommandResponse>, Status> {
        let req = request.into_inner();
        let cmd = mapper::pjob_command_from_proto(req.command)
            .ok_or_else(|| {
                let msg = format!("Invalid command: {}", req.command);
                infra_logging::log_svc_error("execute_command", Some(&req.pjob_id), &msg);
                Status::invalid_argument(msg)
            })?;

        infra_logging::log_svc_request("execute_command", Some(&req.pjob_id));

        match self.store.execute_command(&req.pjob_id, cmd).await {
            Ok(job) => {
                infra_logging::log_svc_response("execute_command", Some(&req.pjob_id), true);
                Ok(Response::new(PJobCommandResponse {
                    status: Some(pb::common::StatusCode {
                        code: 0,
                        message: "OK".to_string(),
                    }),
                    pjob: Some(mapper::process_job_to_proto(job)),
                }))
            }
            Err(CimError::NotFound(e)) => {
                infra_logging::log_svc_error("execute_command", Some(&req.pjob_id), &e);
                Err(Status::not_found(e))
            }
            Err(CimError::InvalidArgument(e)) => {
                infra_logging::log_svc_error("execute_command", Some(&req.pjob_id), &e);
                Err(Status::invalid_argument(e))
            }
            Err(CimError::InvalidStateTransition { from, to }) => {
                let msg = format!("Invalid state transition: {} -> {}", from, to);
                infra_logging::log_svc_error("execute_command", Some(&req.pjob_id), &msg);
                Err(Status::failed_precondition(msg))
            }
            Err(e) => {
                infra_logging::log_svc_error("execute_command", Some(&req.pjob_id), &e.to_string());
                Err(Status::internal(e.to_string()))
            }
        }
    }

    type StreamPJobEventsStream =
        tokio_stream::wrappers::ReceiverStream<Result<pb::common::Event, Status>>;

    async fn stream_p_job_events(
        &self,
        _request: Request<pb::common::Empty>,
    ) -> Result<Response<Self::StreamPJobEventsStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        tokio::spawn(async move {
            let _ = tx.send(Ok(pb::common::Event {
                event_id: "evt-1".to_string(),
                event_name: "PJobCreated".to_string(),
                timestamp: Some(SystemTime::now().into()),
                source: Some(pb::common::ModuleId {
                    equipment_id: "eq-001".to_string(),
                    module_id: "e40-pjob".to_string(),
                    module_type: "service".to_string(),
                }),
                parameters: std::collections::HashMap::new(),
            })).await;
        });
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}
