#![cfg(feature = "integration_tests")]

use std::time::Duration;

use bytes::Bytes;
use figment::{
    providers::Env,
    Figment,
};
use rust_proto::graplinc::grapl::{
    api::{
        client::Connect,
        graph_schema_manager::v1beta1::{
            client::GraphSchemaManagerClient,
            messages as sm_api,
        },
    },
    common::v1beta1::types as common_api,
};

pub fn get_example_graphql_schema() -> Result<Bytes, std::io::Error> {
    // This path is created in rust/Dockerfile
    let path = "/test-fixtures/example_schemas/example.graphql";
    std::fs::read(path).map(Bytes::from)
}

#[tokio::test]
async fn test_deploy_schema() -> eyre::Result<()> {
    let client_config = Figment::new()
        .merge(Env::prefixed("GRAPH_SCHEMA_MANAGER_CLIENT_"))
        .extract()?;
    let mut client = GraphSchemaManagerClient::connect_with_healthcheck(
        client_config,
        Duration::from_secs(60),
        Duration::from_secs(1),
    )
    .await?;

    let tenant_id = uuid::Uuid::new_v4();

    client
        .deploy_schema(sm_api::DeploySchemaRequest {
            tenant_id,
            schema: get_example_graphql_schema()?,
            schema_type: sm_api::SchemaType::GraphqlV0,
            schema_version: 0,
        })
        .await?;

    let edge_schema = client
        .get_edge_schema(sm_api::GetEdgeSchemaRequest {
            tenant_id,
            node_type: common_api::NodeType {
                value: "Process".to_string(),
            },
            edge_name: common_api::EdgeName {
                value: "binary_file".to_string(),
            },
        })
        .await?;

    assert_eq!(edge_schema.reverse_edge_name.value, "executed_as_processes");
    Ok(())
}
