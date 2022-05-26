use clap::Parser;
use organization_management::{
    server::exec_service,
    OrganizationManagementServiceConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let config = OrganizationManagementServiceConfig::parse();
    tracing::info!(message="Started organization-manager", config=?config);

    exec_service(config).await?;
    Ok(())
}
