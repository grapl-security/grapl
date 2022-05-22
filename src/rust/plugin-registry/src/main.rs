use clap::Parser;
use plugin_registry::server::service::{
    exec_service,
    PluginRegistryConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let config = PluginRegistryConfig::parse();
    tracing::info!(message="Starting Plugin Registry Service", config=?config);

    exec_service(config).await?;
    Ok(())
}
