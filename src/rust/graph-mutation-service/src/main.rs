use std::sync::Arc;

use clap::Parser;
use graph_mutation_service::{
    config::GraphMutationServiceConfig,
    graph_mutation::GraphMutationManager,
    reverse_edge_resolver::ReverseEdgeResolver,
};
use rust_proto::graplinc::grapl::api::{
    graph_mutation::v1beta1::server::GraphMutationServiceServer,
    schema_manager::v1beta1::client::SchemaManagerClient,
};
use scylla::CachingSession;
use uid_allocator::client::{
    CachingUidAllocatorServiceClient as CachingUidAllocatorClient,
    UidAllocatorServiceClient as UidAllocatorClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = GraphMutationServiceConfig::parse();
    let mut scylla_config = scylla::SessionConfig::new();
    scylla_config.add_known_nodes_addr(&config.graph_db_config.graph_db_addresses[..]);
    scylla_config.auth_username = Some(config.graph_db_config.graph_db_auth_username.to_owned());
    scylla_config.auth_password = Some(config.graph_db_config.graph_db_auth_password.to_owned());

    let scylla_client = Arc::new(CachingSession::from(
        scylla::Session::connect(scylla_config).await?,
        10_000,
    ));
    let graph_mutation_service = GraphMutationManager::new(
        scylla_client,
        CachingUidAllocatorClient::new(
            UidAllocatorClient::connect(config.uid_allocator_client_config.uid_allocator_address)
                .await?,
            100,
        ),
        ReverseEdgeResolver::new(
            SchemaManagerClient::connect(
                config.schema_manager_client_config.schema_manager_address,
            )
            .await?,
            10_000,
        ),
        1_000_000,
    );

    let (_tx, rx) = tokio::sync::oneshot::channel();
    GraphMutationServiceServer::builder(
        graph_mutation_service,
        config.graph_mutation_service_bind_address,
        rx,
    )
    .build()
    .serve()
    .await?;

    Ok(())
}
