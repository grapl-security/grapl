use plugin_bootstrap::{
    PluginBootstrapServiceConfig,
};
use structopt::StructOpt;
use plugin_bootstrap::server::PluginBootstrapper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let config = PluginBootstrapServiceConfig::from_args();
    tracing::info!(message="Starting Plugin Bootstrap Service", config=?config);

    let plugin_bootstrapper = PluginBootstrapper::load(
        &config.plugin_certificate_path,
        &config.plugin_binary_path,
    )?;

    plugin_bootstrapper.serve(config).await?;

    Ok(())
}

