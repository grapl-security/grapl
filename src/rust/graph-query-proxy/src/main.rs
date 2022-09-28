use clap::Parser;
use grapl_tracing::setup_tracing;
use rust_proto::{
    graplinc::grapl::api::graph_query_service::v1beta1::{
        client::GraphQueryClient,
        server::GraphQueryServiceServer,
    },
    protocol::{
        healthcheck::HealthcheckStatus,
        service_client::ConnectWithConfig,
    },
};
use tokio::net::TcpListener;

use crate::{
    config::GraphQueryProxyConfig,
    server::GraphQueryProxy,
};

mod config;
mod server;

const SERVICE_NAME: &'static str = "graph-query-proxy";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME)?;
    let config = config::GraphQueryProxyConfig::parse();

    let graph_query_service = GraphQueryProxy::new(
        config.tenant_id,
        GraphQueryClient::connect_with_config(config.graph_query_service_client_config.clone())
            .await?,
    );

    exec_service(config, graph_query_service).await
}

#[tracing::instrument(skip(config, api_server))]
pub async fn exec_service(
    config: GraphQueryProxyConfig,
    api_server: GraphQueryProxy,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = config.graph_query_proxy_bind_address;
    let healthcheck_polling_interval_ms = 5000;

    tracing::info!(
        message = "Binding service",
        socket_address = %addr,
    );

    let (server, _shutdown_tx) = GraphQueryServiceServer::new(
        api_server,
        TcpListener::bind(addr.clone()).await?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME: this is garbage
        std::time::Duration::from_millis(healthcheck_polling_interval_ms),
    );

    tracing::info!(message = "starting gRPC server",);

    Ok(server.serve().await?)
}
