use std::time::Duration;

use rust_proto_new::{
    graplinc::grapl::api::plugin_sdk::generators::v1beta1::server::{
        GeneratorApi,
        GeneratorServer,
    },
    protocol::{
        healthcheck::HealthcheckStatus,
        tls::Identity,
    },
};
use structopt::StructOpt;
use tokio::net::TcpListener;

#[derive(StructOpt, Debug)]
pub struct GeneratorServiceConfig {
    #[structopt(env = "PLUGIN_BIND_ADDRESS")]
    pub bind_address: std::net::SocketAddr,
}
impl GeneratorServiceConfig {
    /// An alias for Structopt::from_args, so that consumers don't need to
    /// declare a dependency on structopt
    pub fn from_env_vars() -> Self {
        Self::from_args()
    }
}

pub async fn exec_service(
    graph_generator: impl GeneratorApi + Send + Sync + 'static,
    config: GeneratorServiceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // todo: When bootstrapping and this service are more mature we should determine
    //       the right way to get these configuration values passed around
    let cert = tokio::fs::read("/etc/ssl/private/plugin-client-cert.pem").await?;
    let key = tokio::fs::read("/etc/ssl/private/plugin-client-cert.key").await?;

    let identity = Identity::from_pem(cert, key);

    let healthcheck_polling_interval_ms = 5000; // TODO: un-hardcode
    let (server, _shutdown_tx) = GeneratorServer::new(
        graph_generator,
        TcpListener::bind(config.bind_address.clone()).await?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME: this is garbage
        Duration::from_millis(healthcheck_polling_interval_ms),
        identity,
    );
    tracing::info!(
        message = "starting gRPC server",
        socket_address = %config.bind_address,
    );

    server.serve().await
}
