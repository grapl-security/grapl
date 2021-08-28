use sqlx::postgres::PgPoolOptions;
use grapl_org_management::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::start_server().await?;
    create_db_connection().await?;
    println!("Test");
    Ok(())
}

async fn create_db_connection() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:password@localhost/test").await?;

    // Make a simple query to return the given parameter
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool).await?;

    assert_eq!(row.0, 150);

    Ok(())
}