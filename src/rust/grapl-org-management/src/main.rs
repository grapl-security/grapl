// use sqlx::postgres::PgPoolOptions;
use grapl_org_management::server;
// use sqlx::{Postgres, Pool};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::start_server()
        .await?;

    Ok(())
}
