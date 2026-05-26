use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::sync::RwLock;
use tokio::time::interval;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, warn};

use semi_common::error::CimResult;

use pb::registry::service_registry_server::{ServiceRegistry, ServiceRegistryServer};
use pb::registry::{
    DeregisterRequest, DeregisterResponse, DiscoverRequest, DiscoverResponse,
    HeartbeatRequest, HeartbeatResponse, ListServicesRequest, ListServicesResponse,
    RegisterRequest, RegisterResponse, ServiceEndpoint, ServiceType,
};

// ─────────────────────────────────────────────
// In-memory registry store
// ─────────────────────────────────────────────

#[derive(Debug, Clone)]
struct EndpointRecord {
    endpoint: ServiceEndpoint,
    missed_heartbeats: u32,
}

#[derive(Debug, Clone)]
struct RegistryStore {
    endpoints: Arc<RwLock<HashMap<String, EndpointRecord>>>,
}

impl RegistryStore {
    fn new() -> Self {
        Self {
            endpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn register(&self, endpoint: ServiceEndpoint) -> CimResult<()> {
        let mut endpoints = self.endpoints.write().await;
        let service_id = endpoint.service_id.clone();
        endpoints.insert(
            service_id.clone(),
            EndpointRecord {
                endpoint,
                missed_heartbeats: 0,
            },
        );
        info!(service_id = %service_id, "Service registered");
        Ok(())
    }

    async fn deregister(&self, service_id: &str) -> CimResult<()> {
        let mut endpoints = self.endpoints.write().await;
        endpoints.remove(service_id);
        info!(service_id = %service_id, "Service deregistered");
        Ok(())
    }

    async fn heartbeat(&self, service_id: &str) -> CimResult<()> {
        let mut endpoints = self.endpoints.write().await;
        if let Some(record) = endpoints.get_mut(service_id) {
            record.endpoint.last_heartbeat = Some(std::time::SystemTime::from(Utc::now()).into());
            record.missed_heartbeats = 0;
            record.endpoint.healthy = true;
            Ok(())
        } else {
            Err(semi_common::error::CimError::NotFound(service_id.to_string()))
        }
    }

    async fn discover(
        &self,
        service_type: ServiceType,
        module_id: Option<&str>,
    ) -> Vec<ServiceEndpoint> {
        let endpoints = self.endpoints.read().await;
        endpoints
            .values()
            .filter(|r| {
                r.endpoint.service_type == service_type as i32
                    && r.endpoint.healthy
                    && module_id.map_or(true, |m| r.endpoint.module_id.as_ref().map(|s: &String| s.as_str()) == Some(m))
            })
            .map(|r| r.endpoint.clone())
            .collect()
    }

    async fn list_all(&self) -> Vec<ServiceEndpoint> {
        let endpoints = self.endpoints.read().await;
        endpoints.values().map(|r| r.endpoint.clone()).collect()
    }

    async fn check_health(&self) {
        let mut endpoints = self.endpoints.write().await;
        let mut to_remove = Vec::new();

        for (id, record) in endpoints.iter_mut() {
            record.missed_heartbeats += 1;
            if record.missed_heartbeats >= 3 {
                record.endpoint.healthy = false;
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            warn!(service_id = %id, "Service removed after missed heartbeats");
            endpoints.remove(&id);
        }
    }
}

// ─────────────────────────────────────────────
// gRPC service implementation
// ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RegistryService {
    store: RegistryStore,
}

impl RegistryService {
    fn new() -> Self {
        Self {
            store: RegistryStore::new(),
        }
    }

    async fn start_health_checker(&self) {
        let store = self.store.clone();
        let mut ticker = interval(Duration::from_secs(30));

        tokio::spawn(async move {
            loop {
                ticker.tick().await;
                store.check_health().await;
            }
        });
    }
}

#[tonic::async_trait]
impl ServiceRegistry for RegistryService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        if let Some(endpoint) = req.endpoint {
            self.store
                .register(endpoint)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
        }
        Ok(Response::new(RegisterResponse {
            status: Some(pb::common::StatusCode {
                code: 0,
                message: "OK".to_string(),
            }),
        }))
    }

    async fn deregister(
        &self,
        request: Request<DeregisterRequest>,
    ) -> Result<Response<DeregisterResponse>, Status> {
        let req = request.into_inner();
        self.store
            .deregister(&req.service_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(DeregisterResponse {
            status: Some(pb::common::StatusCode {
                code: 0,
                message: "OK".to_string(),
            }),
        }))
    }

    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatResponse>, Status> {
        let req = request.into_inner();
        self.store
            .heartbeat(&req.service_id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;
        Ok(Response::new(HeartbeatResponse {
            status: Some(pb::common::StatusCode {
                code: 0,
                message: "OK".to_string(),
            }),
        }))
    }

    async fn discover(
        &self,
        request: Request<DiscoverRequest>,
    ) -> Result<Response<DiscoverResponse>, Status> {
        let req = request.into_inner();
        let endpoints = self
            .store
            .discover(req.service_type(), req.module_id.as_ref().map(|s: &String| s.as_str()))
            .await;
        Ok(Response::new(DiscoverResponse {
            status: Some(pb::common::StatusCode {
                code: 0,
                message: "OK".to_string(),
            }),
            endpoints,
        }))
    }

    async fn list_services(
        &self,
        _request: Request<ListServicesRequest>,
    ) -> Result<Response<ListServicesResponse>, Status> {
        let endpoints = self.store.list_all().await;
        Ok(Response::new(ListServicesResponse {
            status: Some(pb::common::StatusCode {
                code: 0,
                message: "OK".to_string(),
            }),
            endpoints,
        }))
    }

    type WatchServicesStream = tokio_stream::wrappers::ReceiverStream<Result<ListServicesResponse, Status>>;

    async fn watch_services(
        &self,
        _request: Request<pb::common::Empty>,
    ) -> Result<Response<Self::WatchServicesStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        let store = self.store.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(5));
            loop {
                ticker.tick().await;
                let endpoints = store.list_all().await;
                let msg = ListServicesResponse {
                    status: Some(pb::common::StatusCode {
                        code: 0,
                        message: "OK".to_string(),
                    }),
                    endpoints,
                };
                if tx.send(Ok(msg)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}

// ─────────────────────────────────────────────
// Main entry point
// ─────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let addr = "[::]:50000".parse()?;
    let service = RegistryService::new();
    service.start_health_checker().await;

    info!("Service Registry listening on {}", addr);

    Server::builder()
        .add_service(ServiceRegistryServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}











