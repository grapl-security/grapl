use plugin_registry::server::exec_service;
use plugin_registry::PluginRegistryServiceConfig;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let opt = PluginRegistryServiceConfig::from_args();
    tracing::info!("Starting Plugin Registry Service");
    exec_service(opt).await?;
    Ok(())
}
