use std::time::Duration;

use clap::Parser;
use plugin_bootstrap::{
    server::{
        PluginBootstrap,
        PluginBootstrapper,
    },
    PluginBootstrapServiceConfig,
};
use rust_proto_new::{
    graplinc::grapl::api::plugin_bootstrap::v1beta1::server::PluginBootstrapServer,
    protocol::healthcheck::HealthcheckStatus,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let config = PluginBootstrapServiceConfig::parse();
    tracing::info!(message="Starting Plugin Bootstrap Service", config=?config);

    let plugin_bootstrapper =
        PluginBootstrapper::load(&config.plugin_certificate_path, &config.plugin_binary_path)?;

    let plugin_bootstrap = PluginBootstrap::new(plugin_bootstrapper);

    let (server, _shutdown_tx) = PluginBootstrapServer::new(
        plugin_bootstrap,
        TcpListener::bind(config.plugin_registry_bind_address).await?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME; this is garbage
        Duration::from_millis(config.plugin_registry_polling_interval_ms),
    );

    Ok(server.serve().await?)
}
