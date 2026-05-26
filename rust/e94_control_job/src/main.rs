use tonic::transport::Server;
use tracing::info;

use e94_control_job::pb::e94::control_job_service_server::ControlJobServiceServer;
use e94_control_job::CJobService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let addr = "[::]:50042".parse()?;
    let service = CJobService::new();

    info!("E94 Control Job Service listening on {}", addr);

    Server::builder()
        .add_service(ControlJobServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
