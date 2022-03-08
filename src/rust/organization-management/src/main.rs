use organization_management::{
    server::exec_service,
    OrganizationManagementServiceConfig,
};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let config = OrganizationManagementServiceConfig::from_args();
    tracing::info!(message="Started organization-manager", config=?config);

    exec_service(config).await?;
    Ok(())
}
