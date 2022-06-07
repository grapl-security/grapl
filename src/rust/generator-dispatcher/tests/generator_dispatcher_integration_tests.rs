#![cfg(feature = "new_integration_tests")]

use std::time::Duration;

use bytes::Bytes;
use rust_proto_new::graplinc::grapl::api::{
    pipeline_ingress::v1beta1::PublishRawLogRequest,
    plugin_work_queue::v1beta1::GetExecuteGeneratorRequest,
};
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

    tracing::info!("creating plugin-work-queue subscriber thread");
    let mut pwq = ctx.plugin_work_queue_client.clone();
    let subscriber = tokio::task::spawn(async move {
        let scanner = async move {
            while let Ok(get_execute_response) = pwq
                .get_execute_generator(GetExecuteGeneratorRequest {})
                .await
            {
                if let Some(job) = get_execute_response.execution_job {
                    if job.tenant_id == tenant_id {
                        return Some(job);
                    }
                } else {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
            None
        };

        tokio::time::timeout(Duration::from_secs(30), scanner)
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

    tracing::info!("waiting for subscriber to complete");
    let matching_job = subscriber
        .instrument(tracing::debug_span!("subscriber"))
        .await
        .expect("could not join subscriber");
    assert!(matching_job.is_some())
}
