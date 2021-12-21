use sqlx::{Pool, Postgres};
use uuid::Uuid;
use tracing::instrument;

#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(type_name = "status_t", rename_all = "lowercase")]
pub enum Status { Enqueued, Failed, Processed }

#[derive(Copy, Clone, Debug, sqlx::Type)]
#[sqlx(transparent)]
pub struct ExecutionId(i64);

#[derive(Clone, Debug)]
pub struct NextExecutionRequest {
    pub execution_key: ExecutionId,
    pub plugin_id: uuid::Uuid,
    pub pipeline_message: Vec<u8>,
    pub trace_id: uuid::Uuid,
}

#[derive(thiserror::Error, Debug)]
pub enum PsqlQueueError {
    #[error("SqlX")]
    SqlX(#[from] sqlx::Error)
}

#[derive(Clone, Debug)]
pub struct Message {
    pub request: NextExecutionRequest,
}

#[derive(Clone, Debug)]
pub struct PsqlQueue {
    pub pool: Pool<Postgres>,
}

impl PsqlQueue {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool,
        }
    }

    #[instrument(skip(pipeline_message), err)]
    pub async fn put_message(
        &self,
        plugin_id: Uuid,
        pipeline_message: Vec<u8>,
        tenant_id: Uuid,
        trace_id: Uuid,
    ) -> Result<(), PsqlQueueError> {
        let now = chrono::Utc::now();
        sqlx::query!(r"
            INSERT INTO plugin_work_queue.plugin_executions (
                plugin_id,
                pipeline_message,
                tenant_id,
                trace_id,
                status,
                creation_time,
                last_updated,
                try_count
            )
            VALUES( $1::UUID, $2, $3::UUID, $4::UUID, 'enqueued', $5, $6, -1 )
        ",
            plugin_id,
            pipeline_message,
            &tenant_id,
            trace_id,
            &now,
            now,
        ).execute(&self.pool)
            .await?;
        Ok(())
    }

    #[instrument(err)]
    pub async fn get_message(&self) -> Result<Option<Message>, PsqlQueueError> {
        let request: Option<NextExecutionRequest> = sqlx::query_as!(
            NextExecutionRequest, r#"
            UPDATE plugin_work_queue.plugin_executions
            SET
                try_count  = plugin_work_queue.plugin_executions.try_count + 1,
                last_updated = CURRENT_TIMESTAMP,
                visible_after  = CURRENT_TIMESTAMP + INTERVAL '10 seconds'
            FROM (
                 SELECT execution_key, plugin_id, pipeline_message, status, creation_time, visible_after, trace_id
                 FROM plugin_work_queue.plugin_executions
                 WHERE status = 'enqueued'
                   AND creation_time >= (CURRENT_TIMESTAMP - INTERVAL '1 day')
                   AND (visible_after IS NULL OR visible_after <= CURRENT_TIMESTAMP)
                 ORDER BY creation_time ASC
                 FOR UPDATE SKIP LOCKED
                 LIMIT 1
             ) AS next_execution
             WHERE plugin_work_queue.plugin_executions.execution_key = next_execution.execution_key
             RETURNING next_execution.execution_key as "execution_key!: ExecutionId", next_execution.plugin_id, next_execution.pipeline_message, next_execution.trace_id
        "#).fetch_optional(&self.pool)
            .await?;

        Ok(request.map(|request| Message { request }))
    }

    #[instrument(skip(output), err)]
    pub async fn ack_success(&self, execution_key: ExecutionId, output: Vec<u8>) -> Result<(), PsqlQueueError> {
        sqlx::query!(
            r#"
                UPDATE plugin_work_queue.plugin_executions
                SET status  = 'processed',
                    execution_result = $2,
                    last_updated = CASE
                        WHEN status != 'processed'
                            THEN CURRENT_TIMESTAMP
                            ELSE last_updated
                        END
                WHERE execution_key = $1 AND execution_result IS NULL
            "#,
            execution_key.0,
            output,
        )
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[instrument(err)]
    pub async fn ack_failure(&self, execution_key: ExecutionId) -> Result<(), PsqlQueueError> {
        sqlx::query!(
            r#"
                UPDATE plugin_work_queue.plugin_executions
                SET status  = 'failed',
                    last_updated = CASE
                        WHEN status != 'processed'
                            THEN CURRENT_TIMESTAMP
                            ELSE last_updated
                        END
                WHERE execution_key = $1
            "#,
            execution_key.0,
        )
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;
    use sqlx::postgres::PgPoolOptions;


    #[derive(Debug)]
    pub struct ExecutionRequest {
        execution_key: ExecutionId,
        plugin_id: uuid::Uuid,
        pipeline_message: Vec<u8>,
        execution_result: Option<Vec<u8>>,
        status: Status,
        creation_time: DateTime<Utc>,
        last_updated: DateTime<Utc>,
        visible_after: Option<DateTime<Utc>>,
        try_count: i32,
    }

    #[tokio::test]
    async fn test_migrate() -> Result<(), Box<dyn std::error::Error>> {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect("postgresql://postgres:password@localhost:5432/postgres").await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_get_and_success() -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!(message="test_get_and_success");
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect("postgresql://postgres:password@localhost:5432/postgres").await?;
        sqlx::migrate!().run(&pool).await?;
        let queue = PsqlQueue { pool };

        // Ensure one message is queued
        queue.put_message(
            uuid::Uuid::new_v4(),
            b"some-message".to_vec(),
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
        ).await?;

        // Retrieve a message
        let msg = queue.get_message().await?
            .expect("No valid message");
        let execution_key = msg.request.execution_key;
        tracing::info!(message="Received message", execution_key=?msg.request.execution_key);
        // Acknowledge the message
        queue.ack_success(execution_key, b"output".to_vec()).await?;

        // The request should be marked as processed
        let row = sqlx::query!(
            r#"SELECT status as "status: Status" FROM plugin_work_queue.plugin_executions WHERE execution_key = $1"#,
            execution_key.0
        )
            .fetch_one(&queue.pool)
            .await?;
        assert_eq!(row.status, Status::Processed);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_get_and_failure() -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!(message="test_get_and_failure");
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect("postgresql://postgres:password@localhost:5432/postgres").await?;

        let queue = PsqlQueue { pool };

        // Ensure one message is queued
        queue.put_message(
            uuid::Uuid::new_v4(),
            b"some-message".to_vec(),
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
        ).await?;

        // Retrieve a message
        let msg = queue.get_message().await?
            .expect("No valid message");
        let execution_key = msg.request.execution_key;
        // Acknowledge the message
        tracing::info!(message="Received message", execution_key=?msg.request.execution_key);
        queue.ack_failure(execution_key).await?;

        // The request should be marked as failed
        let row = sqlx::query!(
            r#"SELECT status as "status: Status" FROM plugin_work_queue.plugin_executions WHERE execution_key = $1"#,
            execution_key.0
        )
            .fetch_one(&queue.pool)
            .await?;
        assert_eq!(row.status, Status::Failed);

        Ok(())
    }
}
