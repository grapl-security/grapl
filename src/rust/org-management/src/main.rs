use org_management::OrgManagementServiceConfig;
use org_management::server::{
    exec_service,
};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let config = OrgManagementServiceConfig::from_args();
    tracing::info!(message="Started org-manager", config=?config);

    exec_service(config).await?;
    Ok(())

}
