#![cfg(feature = "integration_tests")]

// This test needs to be slightly rewritten in the very near (july 2022) future.
// Currently it scans for any new messages showing up for a given `tenant_id`.
// We are removing that field from plugin_work_queue; instead, we should scan
// for new work available for a given `plugin_id`.
// Today, `generator-dispatcher` uses a fake plugin_id; once that's correctly
// implemented, we should change this over to scan for "new work for $plugin_id"

/*
use std::time::Duration;

use bytes::Bytes;
use plugin_work_queue::test_utils::PsqlQueueTestExtensions;
use rust_proto::graplinc::grapl::api::pipeline_ingress::v1beta1::PublishRawLogRequest;
use test_context::test_context;
use tracing::Instrument;
use uuid::Uuid;
mod test_utils;
use test_utils::context::GeneratorDispatcherTestContext;

#[test_context(GeneratorDispatcherTestContext)]
#[tokio::test]
async fn test_dispatcher_inserts_job_into_plugin_work_queue(
    ctx: &mut GeneratorDispatcherTestContext,
) {
    let event_source_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();

    tracing::info!("creating plugin-work-queue scan thread");
    let psql_queue = ctx.plugin_work_queue_psql_client.clone();
    let scan_thread = tokio::task::spawn(async move {
        let scan_for_generator_job = async move {
            while let Ok(generator_messages) =
                psql_queue.get_all_generator_messages(tenant_id).await
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

    let log_event: Bytes = r#"i am some sort of raw log event"#.into();

    tracing::info!("sending publish_raw_log request");
    ctx.pipeline_ingress_client
        .publish_raw_log(PublishRawLogRequest {
            event_source_id,
            tenant_id,
            log_event,
        })
        .await
        .expect("received error response");

    tracing::info!("waiting for scan_thread to complete");
    let matching_job = scan_thread
        .instrument(tracing::debug_span!("scan_thread"))
        .await
        .expect("could not join scan_thread");
    assert!(matching_job.is_some())
}
*/
