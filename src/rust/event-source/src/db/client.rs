use grapl_utils::future_ext::GraplFutureExt;
use sqlx::{
    Pool,
    Postgres,
};

use crate::config::EventSourceDbConfig;

#[derive(Clone, Debug)]
pub struct EventSourceDbClient {
    pub pool: Pool<Postgres>,
}

#[derive(Debug, thiserror::Error)]
pub enum EventSourceDbInitError {
    #[error("Timeout {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),
    #[error("Sqlx {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl EventSourceDbClient {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn try_from(db_config: EventSourceDbConfig) -> Result<Self, EventSourceDbInitError> {
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
}
