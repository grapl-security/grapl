use std::time::Duration;

use rust_proto_new::{
    graplinc::grapl::api::plugin_sdk::generators::v1beta1::{
        GeneratorApi,
        GeneratorServer,
    },
    protocol::{
        healthcheck::HealthcheckStatus,
        tls::Identity,
    },
};
use tokio::net::TcpListener;

pub struct GeneratorServiceConfig {
    pub address: std::net::SocketAddr,
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
        TcpListener::bind(config.address.clone()).await?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME: this is garbage
        Duration::from_millis(healthcheck_polling_interval_ms),
        identity,
    );
    tracing::info!(
        message = "starting gRPC server",
        socket_address = %config.address,
    );

    server.serve().await
}
