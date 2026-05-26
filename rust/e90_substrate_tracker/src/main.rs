use tonic::transport::Server;
use tracing::info;

use e90_substrate_tracker::pb::e90::substrate_tracking_service_server::SubstrateTrackingServiceServer;
use e90_substrate_tracker::TrackingService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let addr = "[::]:50044".parse()?;
    let service = TrackingService::new();

    info!("E90 Substrate Tracking Service listening on {}", addr);

    Server::builder()
        .add_service(SubstrateTrackingServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
