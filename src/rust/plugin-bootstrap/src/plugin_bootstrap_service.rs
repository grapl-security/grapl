use std::time::Duration;

use clap::Parser;
use grapl_tracing::setup_tracing;
use plugin_bootstrap::{
    server::{
        PluginBootstrap,
        PluginBootstrapper,
    },
    PluginBootstrapServiceConfig,
};
use rust_proto::graplinc::grapl::api::{
    plugin_bootstrap::v1beta1::server::PluginBootstrapServer,
    protocol::healthcheck::HealthcheckStatus,
};
use tokio::net::TcpListener;

const SERVICE_NAME: &'static str = "plugin-bootstrap";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME)?;
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
