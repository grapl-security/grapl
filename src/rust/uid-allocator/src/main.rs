use sqlx::PgPool;
use uid_allocator::allocator::UidAllocator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect("postgres://postgres@localhost:5432").await?;

    sqlx::migrate!().run(&pool).await?;

    let _allocator = UidAllocator::new(pool, 100_000, 10_000);
    Ok(())
}
