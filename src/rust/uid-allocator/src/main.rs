use sqlx::PgPool;
use structopt::StructOpt;
use rust_proto_new::graplinc::grapl::api::uid_allocator::v1beta1::server::UidAllocatorServer;
use uid_allocator::allocator::UidAllocator;
use uid_allocator::config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service_config = config::UidAllocatorServiceConfig::from_args();
    service_config.validate()?;
    let pool = PgPool::connect(&service_config.counter_db_config.to_postgres_url()).await?;

    sqlx::migrate!().run(&pool).await?;

    let allocator = UidAllocator::new(
        pool,
        service_config.preallocation_size,
        service_config.maximum_allocation_size,
        service_config.default_allocation_size
    );

    UidAllocatorServer::builder(allocator, service_config.uid_allocator_bind_address).build().serve().await?;

    Ok(())
}
