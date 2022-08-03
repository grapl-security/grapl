#![cfg(feature = "integration_tests")]

use bytes::Bytes;
use e2e_tests::{
    test_fixtures,
    test_utils::{
        context::E2eTestContext,
        predicates::{
            events_36lines_merged_graph_predicate,
            events_36lines_node_identity_predicate,
        },
    },
};
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
use uuid::Uuid;

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
        .contains_for_tenant(tenant_id, |_log: RawLog| true)
        .await?;

    let generated_graphs_scanner =
        KafkaTopicScanner::new(ConsumerConfig::with_topic("generated-graphs"))?
            .contains_for_tenant(tenant_id, |graph: GraphDescription| graph.nodes.len() > 1)
            .await?;

    let node_identifier_scanner =
        KafkaTopicScanner::new(ConsumerConfig::with_topic("identified-graphs"))?
            .contains_for_tenant(tenant_id, events_36lines_node_identity_predicate)
            .await?;

    let graph_merger_scanner = KafkaTopicScanner::new(ConsumerConfig::with_topic("merged-graphs"))?
        .contains_for_tenant(tenant_id, events_36lines_merged_graph_predicate)
        .await?;

    tracing::info!(">> Inserting logs into pipeline-ingress!");

    let log_lines = test_fixtures::get_36_eventlog_xml_separate_lines()?;
    for log_line in log_lines {
        ctx.pipeline_ingress_client
            .publish_raw_log(PublishRawLogRequest {
                event_source_id,
                tenant_id,
                log_event: Bytes::from(log_line),
            })
            .await?;
    }

    tracing::info!(">> Test: that input shows up in raw-logs");

    let _first_raw_log = raw_logs_scanner.get_listen_result().await??;

    tracing::info!(
        ">> Test: `generator-dispatcher` consumes the raw-log and enqueues it in Plugin Work Queue"
    );
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
    let _first_graph = generated_graphs_scanner.get_listen_result().await??;
    // PSA: ^ This likely is picking up output from `sysmon-generator-legacy`,
    // there's no way to currently discriminate between the two paths.

    tracing::info!(">> Test: node-identifier can identify nodes of the unidentified graph, then write to 'identified-graphs'");
    let _identified = node_identifier_scanner.get_listen_result().await??;

    tracing::info!(">> Test: graph-merger wrote these identified nodes to our graph database, then write to 'merged-graphs'");
    let _merged = graph_merger_scanner.get_listen_result().await??;

    // TODO: Perhaps add a test here that looks in dgraph/scylla for those identified nodes

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
    tracing::info!(">> Settting up Event Source, Plugin");

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
