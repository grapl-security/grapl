#![cfg(feature = "integration_tests")]
use rust_proto::graplinc::grapl::api::scylla_provisioner::v1beta1::{
    client::ScyllaProvisionerClient,
    messages::DeployGraphSchemasRequest,
};

type DynError = Box<dyn std::error::Error + Send + Sync>;

// A very very basic smoketest
#[test_log::test(tokio::test)]
async fn test_provision() -> Result<(), DynError> {
    let _span = tracing::info_span!(
        "tenant_id", tenant_id=?tracing::field::Empty,
    );
    tracing::info!("starting test_provision");
    let scylla_provisioner_endpoint = std::env::var("SCYLLA_PROVISIONER_ENDPOINT_ADDRESS")
        .expect("SCYLLA_PROVISIONER_ENDPOINT_ADDRESS");

    tracing::info!(
        scylla_provisioner_endpoint=%scylla_provisioner_endpoint,
        message="connecting to db schema manager service"
    );
    let mut scylla_provisioner_client =
        ScyllaProvisionerClient::connect(scylla_provisioner_endpoint).await?;
    tracing::info!("connected to graph query service");

    let tenant_id = uuid::Uuid::new_v4();
    _span.record("tenant_id", &format!("{tenant_id}"));

    scylla_provisioner_client
        .query_graph_with_uid(DeployGraphSchemasRequest { tenant_id })
        .await?;

    Ok(())
}
