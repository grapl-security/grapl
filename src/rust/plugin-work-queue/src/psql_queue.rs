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
        // `get_message` does a few things
        // 1. It attempts to get a message from the queue
        //      -> Where that message isn't over a day old
        //      -> Where that message is "visible"
        //      -> Where that message isn't currently being evaluated by another transaction
        //      -> Where that message is in the 'enqueued' state
        // 2. Updates the `try_count`
        // 3. Updates the `visible_after`

        // Note that:
        // * messages are invisible for 10 seconds
        // * messages 'expire' after one day
        // * messages currently do not have a maximum retry limit
        // * The one day expiration matches our 1 day partitioning strategy

        // In the future we can leverage a maximum retry limit as well as a batch version of this query
        // A more dynamic visibility strategy would also be reasonable
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

// Pub for testing - otherwise sqlx can't see the query
pub async fn get_status(pool: &sqlx::Pool<Postgres>, execution_key: &ExecutionId) -> Result<Status, sqlx::Error> {
    // The request should be marked as failed
    let row = sqlx::query!(
            r#"SELECT status as "status: Status" FROM plugin_work_queue.plugin_executions WHERE execution_key = $1"#,
            execution_key.0
        )
        .fetch_one(pool)
        .await?;
    Ok(row.status)
}