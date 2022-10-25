use clap::Parser;
use grapl_tracing::setup_tracing;
use organization_management::{
    server::exec_service,
    OrganizationManagementServiceConfig,
};

const SERVICE_NAME: &'static str = "organization-management";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME)?;
    let config = OrganizationManagementServiceConfig::parse();
    tracing::info!(message="Started organization-manager", config=?config);

    exec_service(config).await?;
    Ok(())
}
