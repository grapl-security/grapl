use tracing::info;

use node_identifier::assetdb::{AssetIdDb, AssetIdentifier};
use node_identifier::dynamic_sessiondb::{DynamicMappingDb, DynamicNodeIdentifier};
use node_identifier::sessiondb::SessionDb;
use node_identifier::{ HashCache, NodeIdentifier};
use rusoto_core::Region;
use rusoto_dynamodb::DynamoDbClient;
use grapl_config::env_helpers::FromEnv;

async fn init_local_node_identifier(
    should_default: bool,
) -> Result<NodeIdentifier<DynamoDbClient, HashCache>, Box<dyn std::error::Error>> {
    let cache = HashCache::default();

    info!("region");
    let region = DynamoDbClient::from_env();

    info!("asset_id_db");
    let asset_id_db = AssetIdDb::new(init_dynamodb_client());

    info!("dynamo");
    let dynamo = init_dynamodb_client();
    info!("dyn_session_db");
    let dyn_session_db = SessionDb::new(dynamo.clone(), grapl_config::dynamic_session_table_name());
    info!("dyn_mapping_db");
    let dyn_mapping_db = DynamicMappingDb::new(init_dynamodb_client());
    info!("asset_identifier");
    let asset_identifier = AssetIdentifier::new(asset_id_db);

    info!("dyn_node_identifier");
    let dyn_node_identifier = DynamicNodeIdentifier::new(
        asset_identifier,
        dyn_session_db,
        dyn_mapping_db,
        should_default,
    );

    info!("asset_id_db");
    let asset_id_db = AssetIdDb::new(init_dynamodb_client());

    info!("asset_identifier");
    let asset_identifier = AssetIdentifier::new(asset_id_db);

    info!("asset_id_db");
    let asset_id_db = AssetIdDb::new(init_dynamodb_client());

    info!("node_identifier");

    Ok(NodeIdentifier::new(
        asset_id_db,
        dyn_node_identifier,
        asset_identifier,
        dynamo.clone(),
        should_default,
        cache.clone(),
    ))
}

/// Given:
///     * A Graph with 4 Session-Identifiable Nodes; A, B, C, D
///     * Where A, B share a canonical identity, and C, D share a common identity
///     * Where A, B, have an edge to C, D
/// When:
///     * We identify all nodes
/// Then:
///     * Then, we should have a resulting graph with 2 nodes, and an edge between them

#[tokio::test]
async fn test_service() -> Result<(), Box<dyn std::error::Error>> {
    // todo: I think should_default could in fact be random? It shouldn't matter for this test.
    // let node_identifier = init_local_node_identifier(false).await;

    Ok(())
}
