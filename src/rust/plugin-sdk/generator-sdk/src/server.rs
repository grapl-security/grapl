use std::time::Duration;

use clap::Parser;
use rust_proto::{
    graplinc::grapl::api::plugin_sdk::generators::v1beta1::server::{
        GeneratorApi,
        GeneratorServer,
    },
    protocol::healthcheck::HealthcheckStatus,
};
use tokio::net::TcpListener;

#[derive(clap::Parser, Debug)]
pub struct GeneratorServiceConfig {
    #[clap(long, env = "PLUGIN_BIND_ADDRESS")]
    pub bind_address: std::net::SocketAddr,
}
impl GeneratorServiceConfig {
    /// An alias for clap::parse, so that consumers don't need to
    /// declare a dependency on clap
    pub fn from_env_vars() -> Self {
        Self::parse()
    }
}

pub async fn exec_service(
    graph_generator: impl GeneratorApi + Send + Sync + 'static,
    config: GeneratorServiceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let healthcheck_polling_interval_ms = 5000; // TODO: un-hardcode
    let (server, _shutdown_tx) = GeneratorServer::new(
        graph_generator,
        TcpListener::bind(config.bind_address.clone()).await?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME: this is garbage
        Duration::from_millis(healthcheck_polling_interval_ms),
    );
    tracing::info!(
        message = "starting gRPC server",
        socket_address = %config.bind_address,
    );

    Ok(server.serve().await?)
}
