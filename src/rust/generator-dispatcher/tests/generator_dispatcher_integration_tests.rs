#![cfg(feature = "integration_tests")]

mod test_utils;

use bytes::Bytes;
use plugin_work_queue::{
    test_utils::scan_for_plugin_message_in_pwq,
};
use rust_proto::graplinc::grapl::api::{
    event_source::v1beta1::CreateEventSourceRequest,
    pipeline_ingress::v1beta1::PublishRawLogRequest,
    plugin_registry::v1beta1::{
        PluginMetadata,
        PluginType,
    },
};
use test_context::test_context;
use test_utils::context::GeneratorDispatcherTestContext;
use uuid::Uuid;

#[test_context(GeneratorDispatcherTestContext)]
#[tokio::test]
async fn test_dispatcher_inserts_job_into_plugin_work_queue(
    ctx: &mut GeneratorDispatcherTestContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let display_name = "test_dispatcher_inserts_job_into_plugin_work_queue";
    let tenant_id = Uuid::new_v4();

    // Register an Event Source
    let event_source_id = {
        let create = ctx
            .event_source_client
            .create_event_source(CreateEventSourceRequest {
                display_name: display_name.to_string(),
                description: "arbitrary".to_string(),
                tenant_id,
            })
            .await?;
        create.event_source_id
    };

    // Create a Generator Plugin that responds to that event_source_id
    let plugin_id = {
        let plugin_artifact = futures::stream::once(async { Bytes::from("arbitrary binary") });
        let create = ctx
            .plugin_registry_client
            .create_plugin(
                PluginMetadata {
                    tenant_id,
                    display_name: display_name.to_string(),
                    plugin_type: PluginType::Generator,
                    event_source_id: Some(event_source_id),
                },
                plugin_artifact,
            )
            .await?;
        create.plugin_id
    };

    // Send in the Raw Log Event
    let log_event: Bytes = r#"i am some sort of raw log event"#.into();

    tracing::info!("sending publish_raw_log request");
    ctx.pipeline_ingress_client
        .publish_raw_log(PublishRawLogRequest {
            event_source_id,
            tenant_id,
            log_event,
        })
        .await?;

    let matching_job =
        scan_for_plugin_message_in_pwq(ctx.plugin_work_queue_psql_client.clone(), plugin_id).await;
    assert!(matching_job.is_some());
    Ok(())
}