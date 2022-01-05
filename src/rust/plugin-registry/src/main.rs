use plugin_registry::{
    server::exec_service,
    PluginRegistryServiceConfig,
};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let config = PluginRegistryServiceConfig::from_args();
    tracing::info!(message="Starting Plugin Registry Service", config=?config);

    exec_service(config).await?;
    Ok(())
}
