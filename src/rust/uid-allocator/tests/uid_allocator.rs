#![cfg(all(test, feature = "integration"))]

use clap::Parser;
use rust_proto::graplinc::grapl::api::uid_allocator::v1beta1::client::UidAllocatorClient;
use sqlx::PgPool;
use uid_allocator::client::CachingUidAllocatorClient;

#[tokio::test]
async fn test_uid_allocator() -> Result<(), Box<dyn std::error::Error>> {
    let client_config = uid_allocator::config::UidAllocatorClientConfig::parse();

    sqlx::migrate!().run(&pool).await?;

    let tenant_id = uuid::Uuid::new_v4();

    let endpoint = format!("http://{}", &client_config.uid_allocator_connect_address);
    let mut allocator_client =
        CachingUidAllocatorClient::new(UidAllocatorClient::connect(endpoint).await?, 100);

    allocator_client
        .create_tenant(CreateTenantRequest { tenant_id })
        .await?;

    for i in 1u64..10_000 {
        let next_id = allocator_client.allocate_id(tenant_id).await?;
        assert_eq!(next_id, i);
    }

    Ok(())
}
