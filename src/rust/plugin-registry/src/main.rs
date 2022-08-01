use clap::Parser;
use grapl_tracing::setup_tracing;
use plugin_registry::server::service::{
    exec_service,
    PluginRegistryConfig,
};
const SERVICE_NAME: &'static str = "plugin-registry";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME)?;
    let config = PluginRegistryConfig::parse();
    tracing::info!(message="Starting Plugin Registry Service", config=?config);

    exec_service(config).await?;
    Ok(())
}
