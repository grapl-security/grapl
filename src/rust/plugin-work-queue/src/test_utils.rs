use uuid::Uuid;

use crate::psql_queue::{
    ExecutionId,
    NextExecutionRequest,
    PsqlQueue,
    PsqlQueueError,
};

/// This module is meant to be used by integration tests in other crates.
///
/// ## Example use case
/// So I've introduced a new service 'generator-dispatcher' which:
/// - pops logs off raw-logs
/// - sticks them in plugin-work-queue
///
/// The reasonable integration test for this would, then, be:
/// - insert a log into raw-logs
/// - read jobs in plugin-work-queue to ensure a matching one lives in there
///
/// The problem is this:
/// - We have two concurrent and competing binaries that are calling get_execute_generator:
///   this integration test, and generator-executor.
/// - Generator-executor could potentially mark the job as 'acknowledged', hiding
///   the job from the integration test - it's a race condition.
///
/// As such, I'm exposing a test-only utilities suite that will let a
/// downstream integration test inspect jobs that are already acknowledged, too.

#[async_trait::async_trait]
pub trait PsqlQueueTestExtensions {
    /// Get all generator messages regardless of state
    async fn get_all_generator_messages(
        &self,
        plugin_id: Uuid,
    ) -> Result<Vec<NextExecutionRequest>, PsqlQueueError>;
}

#[async_trait::async_trait]
impl PsqlQueueTestExtensions for PsqlQueue {
    async fn get_all_generator_messages(
        &self,
        plugin_id: Uuid,
    ) -> Result<Vec<NextExecutionRequest>, PsqlQueueError> {
        let generator_messages = sqlx::query_as!(
            NextExecutionRequest,
            r#"
            SELECT
                 execution_key AS "execution_key!: ExecutionId",
                 plugin_id,
                 pipeline_message,
                 tenant_id,
                 trace_id,
                 event_source_id
            FROM plugin_work_queue.generator_plugin_executions
            WHERE plugin_id = $1
            "#,
            plugin_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(generator_messages)
    }
}
