#![cfg(all(test, feature = "integration"))]

use sqlx::PgPool;
use structopt::StructOpt;
use uid_allocator::allocator::UidAllocator;

#[tokio::test]
/// This test allocates 3 times. After the first two allocations, the preallocated space
/// is exhausted.
async fn test_uid_allocator() -> Result<(), Box<dyn std::error::Error>> {
    let counter_db_config = uid_allocator::config::CounterDbConfig::from_args();

    let pool = PgPool::connect(&counter_db_config.to_postgres_url()).await?;

    sqlx::migrate!().run(&pool).await?;

    let tenant_id = uuid::Uuid::new_v4();

    sqlx::query("INSERT INTO counters (tenant_id) VALUES ($1);")
        .bind(tenant_id)
        .execute(&pool)
        .await?;

    let allocator = UidAllocator::new(
        pool,
        20,
        10,
        10,
    );

    let first_allocation = allocator.allocate(tenant_id, 10).await?;
    let second_allocation = allocator.allocate(tenant_id, 10).await?;
    let third_allocation = allocator.allocate(tenant_id, 0).await?;

    // uid `0` is invalid
    assert_ne!(first_allocation.start, 0);
    assert_ne!(second_allocation.start, 0);
    assert_ne!(third_allocation.start, 0);

    // Allocations should never be empty
    assert_ne!(first_allocation.offset, 0);
    assert_ne!(second_allocation.offset, 0);
    assert_ne!(third_allocation.offset, 0);

    // Ensure that the allocations are not overlapping
    assert!(
        (first_allocation.start + (first_allocation.offset as u64)) < second_allocation.start);
    assert!(
        (second_allocation.start + (first_allocation.offset as u64)) < third_allocation.start);

    Ok(())
}