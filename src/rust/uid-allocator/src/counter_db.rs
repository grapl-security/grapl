use grapl_config::PostgresClient;

use crate::{
    allocator::PreAllocation,
    config::CounterDbConfig,
    service::UidAllocatorServiceError,
};

#[derive(Debug)]
pub struct Count {
    pub prev: i64,
    pub new: i64,
}

#[derive(Clone)]
pub struct CounterDb {
    pub pool: sqlx::PgPool,
}

#[async_trait::async_trait]
impl PostgresClient for CounterDb {
    type Config = CounterDbConfig;
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

impl CounterDb {
    pub async fn preallocate(
        &self,
        tenant_id: uuid::Uuid,
        size: u32,
    ) -> Result<PreAllocation, UidAllocatorServiceError> {
        let mut conn = self.pool.acquire().await?;
        // Increments the tenant's allocation counter by `size`, and returns the new value as well as the previous value
        let count = sqlx::query_as!(
            Count,
            "
            UPDATE counters
            SET counter = counter + $1
            FROM (
                     SELECT counter as prev
                     FROM counters
                     WHERE counters.tenant_id = $2
                     LIMIT 1
                     FOR UPDATE
                 ) as c
            WHERE counters.tenant_id = $2
            RETURNING counter as new, c.prev
            ",
            size as i32,
            tenant_id
        )
        .fetch_optional(&mut conn)
        .await?;

        let count = match count {
            Some(count) => count,
            None => return Err(UidAllocatorServiceError::UnknownTenant(tenant_id)),
        };

        Ok(PreAllocation::new(count.prev as u64, count.new as u64))
    }
}
