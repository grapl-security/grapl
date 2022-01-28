use structopt::StructOpt;

use crate::server::server::{
    exec_service,
    PluginRegistryServiceConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let config = PluginRegistryServiceConfig::from_args();
    tracing::info!(message="Starting Plugin Registry Service", config=?config);

    exec_service(config).await?;
    Ok(())
}
