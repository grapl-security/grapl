#![cfg(feature = "integration_tests")]
use rust_proto::graplinc::grapl::api::db_schema_manager::v1beta1::{
    client::DbSchemaManagerClient,
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
    let db_schema_manager_endpoint = std::env::var("DB_SCHEMA_MANAGER_ENDPOINT_ADDRESS")
        .expect("DB_SCHEMA_MANAGER_ENDPOINT_ADDRESS");

    tracing::info!(
        db_schema_manager_endpoint=%db_schema_manager_endpoint,
        message="connecting to db schema manager service"
    );
    let mut db_schema_manager_client =
        DbSchemaManagerClient::connect(db_schema_manager_endpoint).await?;
    tracing::info!("connected to graph query service");

    let tenant_id = uuid::Uuid::new_v4();
    _span.record("tenant_id", &format!("{tenant_id}"));

    db_schema_manager_client
        .query_graph_with_uid(DeployGraphSchemasRequest { tenant_id })
        .await?;

    Ok(())
}
