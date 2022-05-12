use clap::Parser;
use plugin_bootstrap::{
    server::PluginBootstrapper,
    PluginBootstrapServiceConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let config = PluginBootstrapServiceConfig::parse();
    tracing::info!(message="Starting Plugin Bootstrap Service", config=?config);

    let plugin_bootstrapper =
        PluginBootstrapper::load(&config.plugin_certificate_path, &config.plugin_binary_path)?;

    plugin_bootstrapper.serve(config).await?;

    Ok(())
}
