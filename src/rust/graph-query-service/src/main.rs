use std::sync::Arc;

use clap::Parser;
use graph_query_service::{
    config,
    server,
};
use grapl_tracing::setup_tracing;
use rust_proto::graplinc::grapl::api::graph_query_service::v1beta1::server::GraphQueryServiceServer;
use scylla::CachingSession;
use secrecy::ExposeSecret;
use server::GraphQueryService;

use crate::config::GraphQueryServiceConfig;

const SERVICE_NAME: &'static str = "graph-query-service";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME)?;
    let config = GraphQueryServiceConfig::parse();
    let mut scylla_config = scylla::SessionConfig::new();
    scylla_config.add_known_nodes_addr(&config.graph_db_config.graph_db_addresses[..]);
    scylla_config.auth_username = Some(config.graph_db_config.graph_db_auth_username.to_owned());
    scylla_config.auth_password = Some(
        config
            .graph_db_config
            .graph_db_auth_password
            .expose_secret()
            .to_owned(),
    );

    let scylla_client = Arc::new(CachingSession::from(
        scylla::Session::connect(scylla_config).await?,
        10_000,
    ));
    let graph_query_service = GraphQueryService::new(scylla_client);

    let (_tx, rx) = tokio::sync::oneshot::channel();
    GraphQueryServiceServer::builder(
        graph_query_service,
        config.graph_query_service_bind_address,
        rx,
    )
    .build()
    .serve()
    .await?;

    Ok(())
}
