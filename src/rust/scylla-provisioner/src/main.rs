use clap::Parser;
use grapl_tracing::setup_tracing;
use scylla_provisioner::{
    config::ScyllaProvisionerServiceConfig,
    server::exec_service,
};
use tracing::info;

const SERVICE_NAME: &'static str = "scylla-provisioner";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME)?;
    let config = ScyllaProvisionerServiceConfig::parse();
    info!(message="Starting Db Schema Manager Service", config=?config);

    exec_service(config).await?;
    Ok(())
}
