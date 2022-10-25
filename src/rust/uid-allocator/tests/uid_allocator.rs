#![cfg(all(test, feature = "integration_tests"))]

use clap::Parser;
use rust_proto::{
    client_factory::services::UidAllocatorClientConfig,
    graplinc::grapl::api::uid_allocator::v1beta1::messages::CreateTenantKeyspaceRequest,
};
use uid_allocator::client::CachingUidAllocatorServiceClient;

#[tokio::test]
async fn test_uid_allocator() -> eyre::Result<()> {
    let _guard = grapl_tracing::setup_tracing("uid-allocator integ test")?;

    let tenant_id = uuid::Uuid::new_v4();
    tracing::info!(
        tenant_id = ?tenant_id,
    );

    let client_config = UidAllocatorClientConfig::parse();
    let mut allocator_client =
        CachingUidAllocatorServiceClient::from_client_config(client_config, 100).await?;

    tracing::info!("creating keyspace");
    allocator_client
        .create_tenant_keyspace(CreateTenantKeyspaceRequest { tenant_id })
        .await?;

    // If there were 1 single `uid-allocator` instance we're talking to we'd
    // expect this to go monotonically increasing upwards (1 to 10,000),
    // but since we currently have two `uid-allocator` - each reserving 10_000
    // locally on the server - we can make no promises about the order in which
    // uids are allocated.

    // Example behavior:
    // client:  "give me 100 ids for tenant ABC"
    // server1: "ok i've reserved 1-10_000, here's 1-100"
    // client:  "give me 100 ids for tenant ABC"
    // server2: "ok i've reserved 10_001-20_000, here's 10_001-10_101"
    // client:  "give me 100 ids for tenant ABC"
    // server1: "I already have 1-10_000 reserved, so heres 101-200"

    tracing::info!("allocating ids");
    let num_requested_uids: u64 = 11_000;
    // Just a sanity check that we're not allocating insane uids
    let shouldnt_exceed = {
        let num_instances = 2;
        let preallocation_size = 10_000;
        // average case: each uid allocator preallocates 10k and gives ~5k
        // worst case: server1 does 2 preallocations, server2 does 1 allocation
        preallocation_size * (num_instances + 1)
    };
    let mut uids = std::collections::HashSet::with_capacity(num_requested_uids as usize);
    for _ in 0u64..num_requested_uids {
        let next_id = allocator_client.allocate_id(tenant_id).await?;
        assert!(uids.insert(next_id), "next_id {next_id} was not unique");
        assert!(
            next_id <= shouldnt_exceed,
            "next_id {next_id} exceeded expected max {shouldnt_exceed}"
        );
    }

    Ok(())
}
