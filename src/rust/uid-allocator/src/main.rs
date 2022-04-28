#![allow(warnings)]
use sqlx::PgPool;
use uid_allocator::allocator::UidAllocator;

use uid_allocator::service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    let pool =
        PgPool::connect("postgres://postgres@localhost:5432").await?;

    sqlx::migrate!().run(&pool).await?;

    let allocator = UidAllocator::new(
        pool.clone(),
        100_000,
        10_000,
    );
    //
    // let tenant_ids = (0..1000).map(|_| uuid::Uuid::new_v4()).collect::<Vec<_>>();
    // for tenant_id in tenant_ids.iter() {
    //     sqlx::query!(
    //             "INSERT INTO counters (tenant_id, counter) VALUES ($1, $2)",
    //             tenant_id,
    //             1,
    //         ).execute(&allocator.db.pool).await.unwrap();
    // }
    // let mut tenant_ids = tenant_ids.into_iter().cycle();
    //
    // let concurrency = 100_000;
    // let mut ts = Vec::with_capacity(concurrency);
    // let total_allocation_count = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    //
    // for _i in 0..concurrency {
    //     let mut allocator = allocator.clone();
    //     let tenant_id = tenant_ids.next().unwrap();
    //     let total_allocation_count = total_allocation_count.clone();
    //     let t = tokio::task::spawn(async move {
    //
    //         let size = 1_000;
    //
    //         let allocation_count = &std::sync::atomic::AtomicU64::new(0);
    //         let _ = tokio::time::timeout(std::time::Duration::from_secs(10), async move {
    //             loop {
    //                 allocator.allocate(tenant_id, size).await.unwrap();
    //                 allocation_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //                 total_allocation_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //                 tokio::task::yield_now().await;
    //             }
    //         }).await;
    //
    //         let allocation_count = allocation_count.load(std::sync::atomic::Ordering::SeqCst);
    //         println!("{allocation_count},");
    //     });
    //     ts.push(t);
    // }
    //
    // for t in ts {
    //     t.await?;
    // }
    //
    // let total_allocation_count = total_allocation_count.load(std::sync::atomic::Ordering::SeqCst);
    // println!("total: {total_allocation_count}");

    Ok(())
}



