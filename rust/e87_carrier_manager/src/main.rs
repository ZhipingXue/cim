use tonic::transport::Server;
use tracing::info;

use e87_carrier_manager::pb::e87::carrier_management_service_server::CarrierManagementServiceServer;
use e87_carrier_manager::CarrierService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let addr = "[::]:50043".parse()?;
    let service = CarrierService::new();

    info!("E87 Carrier Management Service listening on {}", addr);

    Server::builder()
        .add_service(CarrierManagementServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
