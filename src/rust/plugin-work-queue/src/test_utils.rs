use std::time::Duration;

use tracing::Instrument;
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

    async fn get_all_analyzer_messages(
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

    async fn get_all_analyzer_messages(
        &self,
        plugin_id: Uuid,
    ) -> Result<Vec<NextExecutionRequest>, PsqlQueueError> {
        let analyzer_messages = sqlx::query_as!(
            NextExecutionRequest,
            r#"
            SELECT
                 execution_key AS "execution_key!: ExecutionId",
                 plugin_id,
                 pipeline_message,
                 tenant_id,
                 trace_id,
                 event_source_id
            FROM plugin_work_queue.analyzer_plugin_executions
            WHERE plugin_id = $1
            "#,
            plugin_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(analyzer_messages)
    }
}

pub async fn scan_for_plugin_message_in_pwq(
    psql_queue: PsqlQueue,
    plugin_id: uuid::Uuid,
) -> Option<NextExecutionRequest> {
    tracing::info!("creating plugin-work-queue scan thread");
    let scan_thread = tokio::task::spawn(async move {
        let scan_for_generator_job = async move {
            while let Ok(generator_messages) =
                psql_queue.get_all_generator_messages(plugin_id).await
            {
                if let Some(message) = generator_messages.first() {
                    return Some(message.clone());
                } else {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
            None
        };

        tokio::time::timeout(Duration::from_secs(30), scan_for_generator_job)
            .await
            .expect("failed to consume expected message within 30s")
    });

    tracing::info!("waiting for scan_thread to complete");
    let matching_job = scan_thread
        .instrument(tracing::debug_span!("scan_thread"))
        .await
        .expect("could not join scan_thread");
    matching_job
}

pub async fn scan_analyzer_messages(
    psql_queue: PsqlQueue,
    timeout: Duration,
    analyzer_id: uuid::Uuid,
) -> Option<NextExecutionRequest> {
    tracing::info!("creating plugin-work-queue scan thread");
    let scan_thread = tokio::task::spawn(async move {
        tokio::time::timeout(timeout, async move {
            while let Ok(analyzer_messages) =
                psql_queue.get_all_analyzer_messages(analyzer_id).await
            {
                if let Some(message) = analyzer_messages.first() {
                    return Some(message.clone());
                } else {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }

            None
        })
        .await
        .expect("failed to consume expected message before timeout")
    });

    tracing::info!("waiting for scan_thread to complete");
    scan_thread
        .instrument(tracing::debug_span!("scan_thread"))
        .await
        .expect("could not join scan_thread")
}
