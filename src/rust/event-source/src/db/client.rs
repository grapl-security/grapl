use grapl_utils::future_ext::GraplFutureExt;
use sqlx::{
    Pool,
    Postgres,
};
use tracing::instrument;
use uuid::Uuid;

use super::types::EventSourceRow;
use crate::config::EventSourceDbConfig;

#[derive(Clone, Debug)]
pub struct EventSourceDbClient {
    pub pool: Pool<Postgres>,
}

#[derive(Debug, thiserror::Error)]
pub enum EventSourceDbError {
    #[error("Sqlx {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Timeout {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),
}

impl EventSourceDbClient {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn try_from(db_config: EventSourceDbConfig) -> Result<Self, EventSourceDbError> {
        let postgres_address = format!(
            "postgresql://{}:{}@{}:{}",
            db_config.event_source_db_username,
            db_config.event_source_db_password,
            db_config.event_source_db_hostname,
            db_config.event_source_db_port,
        );

        let pool = sqlx::PgPool::connect(&postgres_address)
            .timeout(std::time::Duration::from_secs(5))
            .await??;

        Ok(Self::new(pool))
    }

    #[instrument(skip(display_name, description, tenant_id), err)]
    pub async fn create_event_source(
        &self,
        display_name: String,
        description: String,
        tenant_id: Uuid,
    ) -> Result<EventSourceRow, EventSourceDbError> {
        let event_source_id = Uuid::new_v4();
        sqlx::query!(
            r"
            INSERT INTO event_sources (
                event_source_id,
                tenant_id,
                display_name,
                description
            )
            VALUES( $1::UUID, $2::UUID, $3, $4 )
        ",
            event_source_id,
            tenant_id,
            display_name,
            description,
        )
        .execute(&self.pool)
        .await?;
        self.get_event_source(event_source_id).await
    }

    pub async fn get_event_source(
        &self,
        event_source_id: Uuid,
    ) -> Result<EventSourceRow, EventSourceDbError> {
        let row = sqlx::query_as!(
            EventSourceRow,
            r#"
            SELECT
                event_source_id,
                tenant_id,
                display_name,
                description,
                created_time,
                last_updated_time,
                active
            FROM event_sources
            WHERE event_source_id = $1
            ;
            "#,
            event_source_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }
}
