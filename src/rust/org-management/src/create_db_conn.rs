use sqlx::{Postgres, Pool};
use sqlx::postgres::PgPoolOptions;

#[tracing::instrument(err)]
pub(crate) async fn create_db_connection() -> Result<Pool<Postgres>, sqlx::Error> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");

    println!("databse url {}", url);

    tracing::info!(message="connecting to postgres", url=%url);
    // Create Connection Pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    // Insert Org Info
    Ok(pool)
}