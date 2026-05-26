use std::sync::Arc;

use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::layer::{Layer, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;

use infra::{FileWriteLayer, GrpcCollectionLayer, NoOpGrpcSender};
use e40_process_job::pb::e40::process_job_service_server::ProcessJobServiceServer;
use e40_process_job::PJobService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ─────────────────────────────────────────────
    // Initialize tracing with multiple layers
    // ─────────────────────────────────────────────

    // 1. Console output (default fmt layer)
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_filter(tracing_subscriber::EnvFilter::from_default_env());

    // 2. File write layer — logs all sm.* and exec.* events at INFO+
    let file_layer = FileWriteLayer::builder("e40-process-job.log")
        .min_level(tracing::Level::INFO)
        .event_filter(|name| name.starts_with("sm.") || name.starts_with("exec.") || name.starts_with("store.") || name.starts_with("svc."))
        .max_size_bytes(10 * 1024 * 1024) // 10MB rotation
        .build();

    // 3. gRPC collection layer — sends WARN+ events to remote service
    // Replace NoOpGrpcSender with your real gRPC client implementation
    let grpc_sender = Arc::new(NoOpGrpcSender);
    let grpc_layer = GrpcCollectionLayer::builder(grpc_sender)
        .min_level(tracing::Level::WARN)
        .event_filter(|name| name.starts_with("sm.") || name.starts_with("exec.") || name.starts_with("svc."))
        .service_name("e40_process_job")
        .build();

    // Combine all layers into the tracing subscriber
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(file_layer)
        .with(grpc_layer)
        .init();

    info!(
        event_name = "svc.initialize",
        service_type = "e40_process_job",
        "E40 Process Job Service starting"
    );

    let addr = "[::]:50041".parse()?;
    let service = PJobService::new();

    info!(
        event_name = "svc.start",
        service_type = "e40_process_job",
        grpc_addr = %addr,
        "E40 Process Job Service listening"
    );

    Server::builder()
        .add_service(ProcessJobServiceServer::new(service))
        .serve(addr)
        .await?;

    info!(
        event_name = "svc.shutdown",
        service_type = "e40_process_job",
        "E40 Process Job Service shutting down"
    );

    Ok(())
}
