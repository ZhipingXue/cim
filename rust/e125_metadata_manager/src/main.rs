use tonic::transport::Server;
use tracing::info;

use e125_metadata_manager::pb::e125::metadata_service_server::MetadataServiceServer;
use e125_metadata_manager::MetadataServiceImpl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let addr = "[::]:50045".parse()?;
    let service = MetadataServiceImpl::new();

    info!("E125 Metadata Manager Service listening on {}", addr);

    Server::builder()
        .add_service(MetadataServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
