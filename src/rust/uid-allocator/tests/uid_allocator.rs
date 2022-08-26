#![cfg(all(test, feature = "integration_tests"))]

use std::collections::HashSet;

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

    tracing::info!("allocating ids");
    let mut uids = HashSet::with_capacity(11000);
    for _ in 0u64..11000 {
        let next_id = allocator_client.allocate_id(tenant_id).await?;
        assert!(uids.insert(next_id), "next_id was not unique");
    }

    Ok(())
}
