#![cfg(all(test, feature = "integration_tests"))]

use clap::Parser;
use rust_proto::graplinc::grapl::api::uid_allocator::v1beta1::{
    client::UidAllocatorServiceClient,
    messages::CreateTenantKeyspaceRequest,
};
use uid_allocator::client::CachingUidAllocatorServiceClient;

#[tokio::test]
async fn test_uid_allocator() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = grapl_tracing::setup_tracing("uid-allocator integ test")?;
    let client_config = uid_allocator::config::UidAllocatorClientConfig::parse();

    let tenant_id = uuid::Uuid::new_v4();
    let endpoint = client_config.uid_allocator_connect_address;
    tracing::info!(
        tenant_id = ?tenant_id,
        endpoint = %endpoint,
    );

    let mut allocator_client = CachingUidAllocatorServiceClient::new(
        UidAllocatorServiceClient::connect(endpoint).await?,
        100,
    );

    tracing::info!("creating keyspace");
    allocator_client
        .create_tenant_keyspace(CreateTenantKeyspaceRequest { tenant_id })
        .await?;


    // If there were 1 single `uid-allocator` instance we're talking to we'd
    // expect this to go monotonically increasing upwards (1 to 10,000),
    // but since we currently have two instances - each reserving a large space
    // locally on the server - we can make no promises about the order in which
    // uids are allocated. 
    tracing::info!("allocating ids");
    let mut last_id = 0;
    for _ in 1u64..10000 {
        let next_id = allocator_client.allocate_id(tenant_id).await?;
        assert!(next_id > last_id);
        last_id = next_id;
    }
    tracing::info!(last_id = last_id);

    Ok(())
}
