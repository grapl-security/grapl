use grapl_config::PostgresClient;

use crate::config::SchemaDbConfig;

pub struct SchemaDbClient {
    pool: sqlx::PgPool,
}

#[async_trait::async_trait]
impl PostgresClient for SchemaDbClient {
    type Config = SchemaDbConfig;
    type Error = grapl_config::PostgresDbInitError;

    fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Self { pool }
    }

    #[tracing::instrument]
    async fn migrate(pool: &sqlx::Pool<sqlx::Postgres>) -> Result<(), sqlx::migrate::MigrateError> {
        tracing::info!(message = "Performing database migration");

        sqlx::migrate!().run(pool).await
    }
}

impl SchemaDbClient {}
