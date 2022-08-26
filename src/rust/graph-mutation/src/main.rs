use std::{
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};

use clap::Parser;
use graph_mutation::{
    config::GraphMutationServiceConfig,
    graph_mutation::GraphMutationManager,
    reverse_edge_resolver::ReverseEdgeResolver,
};
use rust_proto::{
    client_factory::build_grpc_client,
    graplinc::grapl::api::graph_mutation::v1beta1::server::GraphMutationServer,
    protocol::healthcheck::HealthcheckStatus,
};
use scylla::CachingSession;
use tokio::net::TcpListener;
use uid_allocator::client::{
    CachingUidAllocatorServiceClient as CachingUidAllocatorClient,
    UidAllocatorServiceClient as UidAllocatorClient,
};

const SERVICE_NAME: &'static str = "graph-mutation";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = grapl_tracing::setup_tracing(SERVICE_NAME);

    let config = GraphMutationServiceConfig::parse();
    let mut scylla_config = scylla::SessionConfig::new();
    scylla_config.add_known_nodes_addr(&config.graph_db_config.graph_db_addresses[..]);
    scylla_config.auth_username = Some(config.graph_db_config.graph_db_auth_username.to_owned());
    scylla_config.auth_password = Some(config.graph_db_config.graph_db_auth_password.to_owned());

    let scylla_client = Arc::new(CachingSession::from(
        scylla::Session::connect(scylla_config).await?,
        10_000,
    ));
    let graph_schema_manager_client =
        build_grpc_client(config.graph_schema_manager_client_config).await?;
    let uid_allocator_client =
        CachingUidAllocatorClient::from_client_config(config.uid_allocator_client_config, 100)
            .await?;
    let graph_mutation_service = GraphMutationManager::new(
        scylla_client,
        uid_allocator_client,
        ReverseEdgeResolver::new(graph_schema_manager_client, 10_000),
        1_000_000,
    );
    exec_service(config.graph_mutation_bind_address, graph_mutation_service).await
}

#[tracing::instrument(skip(addr, api_server))]
pub async fn exec_service(
    addr: SocketAddr,
    api_server: GraphMutationManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let healthcheck_polling_interval_ms = 5000;

    tracing::info!(
        message = "Binding service",
        socket_address = %addr,
    );

    let (server, _shutdown_tx) = GraphMutationServer::new(
        api_server,
        TcpListener::bind(addr.clone()).await?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME: this is garbage
        Duration::from_millis(healthcheck_polling_interval_ms),
    );

    tracing::info!(message = "starting gRPC server",);

    Ok(server.serve().await?)
}
