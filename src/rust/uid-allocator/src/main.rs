use clap::Parser;
use uid_allocator::config::UidAllocatorServiceConfig;

const SERVICE_NAME: &str = "uid-allocator";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = grapl_tracing::setup_tracing(SERVICE_NAME)?;
    let config = UidAllocatorServiceConfig::parse();
    tracing::info!(message="Starting Uid Allocator Service", config=?config);

    uid_allocator::service::exec_service(config).await?;

    Ok(())
}
