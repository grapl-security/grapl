#![cfg(feature = "integration_tests")]
mod test_utils;

use kafka::{
    config::ConsumerConfig,
    test_utils::topic_scanner::KafkaTopicScanner,
};
use plugin_work_queue::test_utils::scan_for_plugin_message_in_pwq;
use rust_proto::graplinc::grapl::{
    api::{
        event_source::v1beta1::CreateEventSourceRequest,
        graph::v1beta1::GraphDescription,
        pipeline_ingress::v1beta1::PublishRawLogRequest,
        plugin_registry::v1beta1::{
            DeployPluginRequest,
            PluginMetadata,
            PluginType,
        },
    },
    pipeline::v1beta1::RawLog,
};
use test_context::test_context;
use test_utils::context::E2eTestContext;
use uuid::Uuid;

use crate::test_utils::predicates::events6_node_identity_predicate;

mod test_fixtures {
    use bytes::Bytes;
    use grapl_utils::iter_ext::GraplIterExt;

    /// Kafka messages in our pipeline have a max size of 1MB;
    /// so we just send N lines at a time.
    pub fn get_events6_xml_chunked() -> Result<Vec<Bytes>, std::io::Error> {
        let chunk_size = 500;

        let filename = "/test-fixtures/events6.xml"; // This path is created in rust/Dockerfile
        let file = String::from_utf8(std::fs::read(filename)?).unwrap();
        let lines = file.split("\n");
        let line_chunks = lines.into_iter().chunks_owned(chunk_size);
        let byte_chunks = line_chunks
            .into_iter()
            .map(|lines| Bytes::from(lines.join("\n")))
            .collect();
        Ok(byte_chunks)
    }

    pub fn get_sysmon_generator() -> Result<Bytes, std::io::Error> {
        std::fs::read("/test-fixtures/sysmon-generator").map(Bytes::from)
    }
}

#[test_context(E2eTestContext)]
#[tokio::test]
async fn test_sysmon_log_e2e(ctx: &mut E2eTestContext) -> Result<(), Box<dyn std::error::Error>> {
    let test_name = "test_sysmon_log_e2e";
    let tenant_id = Uuid::new_v4();

    let SetupResult {
        tenant_id,
        plugin_id,
        event_source_id,
    } = common_setup(ctx, test_name, tenant_id).await?;

    tracing::info!(">> Setup complete. Now let's test milestones in the pipeline.");

    let raw_logs_scanner = KafkaTopicScanner::new(ConsumerConfig::with_topic("raw-logs"))?
        .contains_for_tenant(tenant_id, |_log: &RawLog| true)
        .await?;

    // Right now:
    // - sysmon-generator-legacy produces messages with correct tenant ID
    // - new plugin setup emits stuff with tenant id Uuid::nil (until we kill legacy)
    let special_generated_graphs_tenant_id = Uuid::nil();
    let generated_graphs_scanner =
        KafkaTopicScanner::new(ConsumerConfig::with_topic("generated-graphs"))?
            .contains_for_tenant(
                special_generated_graphs_tenant_id,
                |graph: &GraphDescription| graph.nodes.len() > 1,
            )
            .await?;

    let node_identifier_scanner =
        KafkaTopicScanner::new(ConsumerConfig::with_topic("identified-graphs"))?
            .contains_for_tenant(
                special_generated_graphs_tenant_id,
                events6_node_identity_predicate,
            )
            .await?;

    tracing::info!(">> Inserting logs into pipeline-ingress!");

    let log_chunks = test_fixtures::get_events6_xml_chunked()?;
    for log_chunk in log_chunks {
        tracing::info!(message = "Uploading a chunk", len = log_chunk.len());
        ctx.pipeline_ingress_client
            .publish_raw_log(PublishRawLogRequest {
                event_source_id,
                tenant_id,
                log_event: log_chunk,
            })
            .await?;
    }

    tracing::info!(">> Testing that input shows up in raw-logs");

    let _first_raw_log = raw_logs_scanner.get_listen_result().await??;

    tracing::info!(">> Testing that `generator-dispatcher` consumes the raw-log and enqueues it in Plugin Work Queue");
    {
        let msg =
            scan_for_plugin_message_in_pwq(ctx.plugin_work_queue_psql_client.clone(), plugin_id)
                .await;
        assert!(msg.is_some());
    }

    // Then, the generator-execution-sidecar will pull this PWQ message and
    // send it to the Generator.
    // After the Generator is done, the generator-execution-sidecar will tell
    // Plugin Work Queue to write to the "generated-graphs" topic.
    tracing::info!(">> Testing that the generator eventually writes to `generated-graphs`");
    let first_graph = generated_graphs_scanner.get_listen_result().await??;
    tracing::debug!(
        message = "first_graph",
        expected_tenant_id = ?tenant_id,
        graph = ?first_graph,
    );

    tracing::info!(">> Testing that node-identifier can identify nodes of the generated graph");
    let _identified = node_identifier_scanner.get_listen_result().await??;

    Ok(())
}

pub struct SetupResult {
    tenant_id: Uuid,
    plugin_id: Uuid,
    event_source_id: Uuid,
}
async fn common_setup(
    ctx: &mut E2eTestContext,
    test_name: &str,
    tenant_id: Uuid,
) -> Result<SetupResult, Box<dyn std::error::Error>> {
    tracing::info!(">> Settting up");

    // Register an Event Source
    let event_source = ctx
        .event_source_client
        .create_event_source(CreateEventSourceRequest {
            display_name: test_name.to_string(),
            description: "arbitrary".to_string(),
            tenant_id,
        })
        .await?;

    // Deploy a Generator Plugin that responds to that event_source_id
    let plugin = {
        let plugin_artifact = test_fixtures::get_sysmon_generator()?;
        let plugin = ctx
            .plugin_registry_client
            .create_plugin(
                PluginMetadata {
                    tenant_id,
                    display_name: test_name.to_string(),
                    plugin_type: PluginType::Generator,
                    event_source_id: Some(event_source.event_source_id),
                },
                futures::stream::once(async move { plugin_artifact }),
            )
            .await?;

        ctx.plugin_registry_client
            .deploy_plugin(DeployPluginRequest {
                plugin_id: plugin.plugin_id,
            })
            .await?;
        plugin
    };

    Ok(SetupResult {
        tenant_id,
        plugin_id: plugin.plugin_id,
        event_source_id: event_source.event_source_id,
    })
}
