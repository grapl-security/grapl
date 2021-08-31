use sqlx::postgres::PgPoolOptions;
use grapl_org_management::server;
use sqlx::{Postgres, Pool};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::start_server().await?;

    let row: (i64, ) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool).await?;

    assert_eq!(row.0, 150);

    println!("Test");
    Ok(())
}
