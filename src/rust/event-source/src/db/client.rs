use grapl_config::PostgresClient;
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

#[async_trait::async_trait]
impl PostgresClient for EventSourceDbClient {
    type Config = EventSourceDbConfig;
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

impl EventSourceDbClient {
    #[instrument(skip(display_name, description, tenant_id), err)]
    pub async fn create_event_source(
        &self,
        display_name: String,
        description: String,
        tenant_id: Uuid,
    ) -> Result<EventSourceRow, sqlx::Error> {
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

    #[instrument(skip(event_source_id, display_name, description, active), err)]
    pub async fn update_event_source(
        &self,
        event_source_id: Uuid,
        display_name: String,
        description: String,
        active: bool,
    ) -> Result<EventSourceRow, sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE event_sources
            SET 
                display_name = $1,
                description = $2,
                active = $3,
                last_updated_time = CURRENT_TIMESTAMP
            WHERE
                event_source_id = $4
            "#,
            display_name,
            description,
            active,
            event_source_id,
        )
        .fetch_optional(&self.pool)
        .await?;
        self.get_event_source(event_source_id).await
    }

    pub async fn get_event_source(
        &self,
        event_source_id: Uuid,
    ) -> Result<EventSourceRow, sqlx::Error> {
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
