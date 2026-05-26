pub mod pb {
    pub mod common {
        tonic::include_proto!("cim.common");
    }
    pub mod e134 {
        tonic::include_proto!("cim.e134");
    }
}

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

use pb::e134::data_collection_service_server::{DataCollectionService, DataCollectionServiceServer};
use pb::e134::{
    CreatePlanRequest, CreatePlanResponse, DataCollectionPlan, DataReport, DcpState,
    GetPlanRequest, GetPlanResponse, ListPlansRequest, ListPlansResponse,
    PlanCommand, PlanCommandRequest, PlanCommandResponse,
};

#[derive(Debug, Clone)]
pub struct DcpStore {
    plans: Arc<RwLock<HashMap<String, DataCollectionPlan>>>,
}

impl DcpStore {
    pub fn new() -> Self {
        Self {
            plans: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create(
        &self,
        plan: DataCollectionPlan,
    ) -> Result<DataCollectionPlan, String> {
        let mut plans = self.plans.write().await;
        if plans.contains_key(&plan.plan_id) {
            return Err(format!("Plan {} already exists", plan.plan_id));
        }
        plans.insert(plan.plan_id.clone(), plan.clone());
        info!(plan_id = %plan.plan_id, "Data Collection Plan created");
        Ok(plan)
    }

    pub async fn get(&self, plan_id: &str) -> Option<DataCollectionPlan> {
        let plans = self.plans.read().await;
        plans.get(plan_id).cloned()
    }

    pub async fn list(&self) -> Vec<DataCollectionPlan> {
        let plans = self.plans.read().await;
        plans.values().cloned().collect()
    }

    pub async fn execute_command(
        &self,
        plan_id: &str,
        command: PlanCommand,
    ) -> Option<DataCollectionPlan> {
        let mut plans = self.plans.write().await;
        if let Some(plan) = plans.get_mut(plan_id) {
            let new_state = match command {
                PlanCommand::Activate => DcpState::Activated,
                PlanCommand::Pause => DcpState::Paused,
                PlanCommand::Resume => DcpState::Activated,
                PlanCommand::Deactivate => DcpState::Deactivated,
                PlanCommand::Delete => DcpState::Deleted,
                _ => return Some(plan.clone()),
            };
            plan.state = new_state as i32;
            if command == PlanCommand::Activate {
                plan.activated_at = Some(std::time::SystemTime::from(Utc::now()).into());
            }
            info!(plan_id = %plan_id, state = ?new_state, "DCP state updated");
            Some(plan.clone())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct DcpService {
    store: DcpStore,
}

impl DcpService {
    pub fn new() -> Self {
        Self {
            store: DcpStore::new(),
        }
    }
}

#[tonic::async_trait]
impl DataCollectionService for DcpService {
    async fn create_plan(
        &self,
        request: Request<CreatePlanRequest>,
    ) -> Result<Response<CreatePlanResponse>, Status> {
        let req = request.into_inner();
        let plan_id = format!("plan-{}", Utc::now().timestamp_millis());
        let plan = DataCollectionPlan {
            plan_id: plan_id.clone(),
            plan_name: req.plan_name,
            state: DcpState::Created as i32,
            parameters: req.parameters,
            created_at: Some(std::time::SystemTime::from(Utc::now()).into()),
            activated_at: None,
        };

        match self.store.create(plan).await {
            Ok(plan) => Ok(Response::new(CreatePlanResponse {
                status: Some(pb::common::StatusCode {
                    code: 0,
                    message: "OK".to_string(),
                }),
                plan: Some(plan),
            })),
            Err(e) => Err(Status::already_exists(e)),
        }
    }

    async fn execute_command(
        &self,
        request: Request<PlanCommandRequest>,
    ) -> Result<Response<PlanCommandResponse>, Status> {
        let req = request.into_inner();
        match self.store.execute_command(&req.plan_id, req.command()).await {
            Some(plan) => Ok(Response::new(PlanCommandResponse {
                status: Some(pb::common::StatusCode {
                    code: 0,
                    message: "OK".to_string(),
                }),
                plan: Some(plan),
            })),
            None => Err(Status::not_found(format!("Plan {} not found", req.plan_id))),
        }
    }

    async fn get_plan(
        &self,
        request: Request<GetPlanRequest>,
    ) -> Result<Response<GetPlanResponse>, Status> {
        let req = request.into_inner();
        match self.store.get(&req.plan_id).await {
            Some(plan) => Ok(Response::new(GetPlanResponse {
                status: Some(pb::common::StatusCode {
                    code: 0,
                    message: "OK".to_string(),
                }),
                plan: Some(plan),
            })),
            None => Err(Status::not_found(format!("Plan {} not found", req.plan_id))),
        }
    }

    async fn list_plans(
        &self,
        _request: Request<ListPlansRequest>,
    ) -> Result<Response<ListPlansResponse>, Status> {
        let plans = self.store.list().await;
        Ok(Response::new(ListPlansResponse { plans }))
    }

    type StreamReportsStream =
        tokio_stream::wrappers::ReceiverStream<Result<DataReport, Status>>;

    async fn stream_reports(
        &self,
        _request: Request<pb::common::Empty>,
    ) -> Result<Response<Self::StreamReportsStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        tokio::spawn(async move {
            let _ = tx.send(Ok(DataReport {
                plan_id: "plan-1".to_string(),
                report_id: "rpt-1".to_string(),
                timestamp: Some(std::time::SystemTime::from(Utc::now()).into()),
                values: Default::default(),
            })).await;
        });
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let addr = "[::]:50046".parse()?;
    let service = DcpService::new();

    tracing::info!("E134 Data Collection Service listening on {}", addr);

    Server::builder()
        .add_service(DataCollectionServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

