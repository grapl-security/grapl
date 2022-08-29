#![cfg(feature = "integration_tests")]

use clap::Parser;
use rust_proto::{
    client_factory::{
        build_grpc_client,
        services::ScyllaProvisionerClientConfig,
    },
    graplinc::grapl::api::scylla_provisioner::v1beta1::messages::DeployGraphSchemasRequest,
};

type DynError = Box<dyn std::error::Error + Send + Sync>;

// A very very basic smoketest
#[test_log::test(tokio::test)]
async fn test_provision() -> Result<(), DynError> {
    let _span = tracing::info_span!(
        "tenant_id", tenant_id=?tracing::field::Empty,
    );
    tracing::info!("starting test_provision");

    tracing::info!(message = "connecting to db schema manager service");
    let mut scylla_provisioner_client =
        build_grpc_client(ScyllaProvisionerClientConfig::parse()).await?;
    tracing::info!("connected to graph query service");

    let tenant_id = uuid::Uuid::new_v4();
    _span.record("tenant_id", &format!("{tenant_id}"));

    scylla_provisioner_client
        .deploy_graph_schemas(DeployGraphSchemasRequest { tenant_id })
        .await?;

    Ok(())
}
