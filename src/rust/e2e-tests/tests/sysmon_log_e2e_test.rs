#![cfg(feature = "integration_tests")]

use std::time::Duration;

use bytes::Bytes;
use e2e_tests::{
    test_fixtures,
    test_utils::{
        context::{
            E2eTestContext,
            SetupResult,
        },
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
        graph::v1beta1::{
            GraphDescription,
            IdentifiedGraph,
            MergedGraph,
        },
        pipeline_ingress::v1beta1::PublishRawLogRequest,
    },
    pipeline::v1beta1::{
        Envelope,
        RawLog,
    },
};
use test_context::test_context;

#[tracing::instrument(skip(ctx))]
#[test_context(E2eTestContext)]
#[tokio::test]
async fn test_sysmon_log_e2e(ctx: &mut E2eTestContext) -> Result<(), Box<dyn std::error::Error>> {
    let test_name = "test_sysmon_log_e2e";

    let SetupResult {
        tenant_id,
        plugin_id,
        event_source_id,
    } = ctx.setup_sysmon_generator(test_name).await?;

    tracing::info!(">> Setup complete. Now let's test milestones in the pipeline.");

    let raw_logs_scanner_handle = KafkaTopicScanner::new(
        ConsumerConfig::with_topic("raw-logs"),
        Duration::from_secs(60),
    )
    .scan_for_tenant(tenant_id, 36, |_log: RawLog| true)
    .await; // this blocks for 10s

    let generated_graphs_scanner_handle = KafkaTopicScanner::new(
        ConsumerConfig::with_topic("generated-graphs"),
        Duration::from_secs(60),
    )
    .scan_for_tenant(tenant_id, 360, |_graph: GraphDescription| true)
    .await; // this blocks for 10s

    let node_identifier_scanner_handle = KafkaTopicScanner::new(
        ConsumerConfig::with_topic("identified-graphs"),
        Duration::from_secs(60),
    )
    .scan_for_tenant(tenant_id, 360, |_graph: IdentifiedGraph| true)
    .await; // this blocks for another 10s

    let graph_merger_scanner_handle = KafkaTopicScanner::new(
        ConsumerConfig::with_topic("merged-graphs"),
        Duration::from_secs(60),
    )
    .scan_for_tenant(tenant_id, 360, |_graph: MergedGraph| true)
    .await; // and finally this blocks for another 10s

    // Adding up all of the above, we have 40+s of blocking to get to this
    // point. So the timeouts above need to each be at least 40s. See the
    // warning in the docs for KafkaTopicScanner.scan().
    // TODO: get those 40s back by running these all concurrently. This should
    // allow us to reduce the timeouts to ~30s which will give significant gains
    // in total test time.

    tracing::info!(">> Inserting logs into pipeline-ingress!");

    let log_lines = test_fixtures::get_36_eventlog_xml_separate_lines()?;
    for log_line in &log_lines {
        ctx.pipeline_ingress_client
            .publish_raw_log(PublishRawLogRequest {
                event_source_id,
                tenant_id,
                log_event: Bytes::from(log_line.clone()),
            })
            .await?;
    }

    tracing::info!(">> Test: that input shows up in raw-logs");

    let raw_logs = raw_logs_scanner_handle.await?;
    assert_eq!(raw_logs.len(), 36);
    assert_eq!(raw_logs.len(), log_lines.len());
    assert!(raw_logs.iter().all(|envelope| {
        envelope.tenant_id() == tenant_id && envelope.event_source_id() == event_source_id
    }));

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
    let generated_graphs = generated_graphs_scanner_handle.await?;
    assert!(!generated_graphs.is_empty());

    tracing::info!(">> Test: node-identifier can identify nodes of the unidentified graph, then write to 'identified-graphs'");
    let identified_graphs = node_identifier_scanner_handle.await?;
    assert!(!identified_graphs.is_empty());

    let filtered_identified_graphs = identified_graphs
        .iter()
        .cloned()
        .filter(move |envelope| {
            let envelope = envelope.clone();
            let identified_graph = envelope.inner_message();
            events_36lines_node_identity_predicate(identified_graph)
        })
        .collect::<Vec<Envelope<IdentifiedGraph>>>();

    assert!(!filtered_identified_graphs.is_empty());
    assert_eq!(filtered_identified_graphs.len(), 1);

    tracing::info!(">> Test: graph-merger wrote these identified nodes to our graph database, then write to 'merged-graphs'");
    let merged_graphs = graph_merger_scanner_handle.await?;
    assert!(!merged_graphs.is_empty());

    let filtered_merged_graphs = merged_graphs
        .iter()
        .cloned()
        .filter(move |envelope| {
            let envelope = envelope.clone();
            let merged_graph = envelope.inner_message();
            events_36lines_merged_graph_predicate(merged_graph)
        })
        .collect::<Vec<Envelope<MergedGraph>>>();

    assert!(!filtered_merged_graphs.is_empty());
    assert_eq!(filtered_merged_graphs.len(), 1);

    // TODO: Perhaps add a test here that looks in dgraph/scylla for those identified nodes

    Ok(())
}
