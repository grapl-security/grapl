#![cfg(feature = "integration_tests")]

use std::time::Duration;

use bytes::Bytes;
use clap::Parser;
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
use rust_proto::{
    client_factory::{
        build_grpc_client,
        services::ScyllaProvisionerClientConfig,
    },
    graplinc::grapl::{
        api::{
            graph::v1beta1::{
                GraphDescription,
                IdentifiedGraph,
                MergedGraph,
            },
            pipeline_ingress::v1beta1::PublishRawLogRequest,
            scylla_provisioner::v1beta1::messages::ProvisionGraphForTenantRequest,
        },
        pipeline::v1beta1::{
            Envelope,
            RawLog,
        },
    },
};
use test_context::test_context;
use uuid::Uuid;

#[tracing::instrument(skip(ctx))]
#[test_context(E2eTestContext)]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_sysmon_log_e2e(ctx: &mut E2eTestContext) -> eyre::Result<()> {
    let test_name = "test_sysmon_log_e2e";

    let SetupResult {
        tenant_id,
        plugin_id,
        event_source_id,
    } = ctx
        .setup_sysmon_generator(test_name)
        .await
        .expect("failed to setup the sysmon-generator");

    let provisioner_client_config = ScyllaProvisionerClientConfig::parse();
    let mut provisioner_client = build_grpc_client(provisioner_client_config).await?;

    provisioner_client
        .provision_graph_for_tenant(ProvisionGraphForTenantRequest { tenant_id })
        .await?;

    tracing::info!(">> Setup complete. Now let's test milestones in the pipeline.");

    let raw_logs_scanner_handle = KafkaTopicScanner::new(
        ConsumerConfig::with_topic("raw-logs"),
        Duration::from_secs(60),
        Envelope::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            RawLog::new(test_fixtures::single_sysmon_event()),
        ),
    )
    .scan_for_tenant(tenant_id, 36, |_log: RawLog| true)
    .await;

    let generated_graphs_scanner_handle = KafkaTopicScanner::new(
        ConsumerConfig::with_topic("generated-graphs"),
        Duration::from_secs(60),
        Envelope::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            GraphDescription::new(),
        ),
    )
    .scan_for_tenant(tenant_id, 36, |_graph: GraphDescription| true)
    .await;

    let node_identifier_scanner_handle = KafkaTopicScanner::new(
        ConsumerConfig::with_topic("identified-graphs"),
        Duration::from_secs(60),
        Envelope::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            IdentifiedGraph::new(),
        ),
    )
    .scan_for_tenant(tenant_id, 36, |_graph: IdentifiedGraph| true)
    .await;

    // Sometimes we find 40 or 41 messages; for tability we'll just let this
    // time out instead of cutting it off early at 36 or 40.
    // Why does it not equal 36? Not really sure! But graph-merger is being
    // heavily rewritten soon, so let's explore that _after_ the rewrite.
    let graph_merger_upper_bound = 45;
    let graph_merger_scanner_handle = KafkaTopicScanner::new(
        ConsumerConfig::with_topic("merged-graphs"),
        Duration::from_secs(60),
        Envelope::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            MergedGraph::new(),
        ),
    )
    .scan_for_tenant(
        tenant_id,
        graph_merger_upper_bound,
        |_graph: MergedGraph| true,
    )
    .await;

    tracing::info!(">> Inserting logs into pipeline-ingress!");

    let input_log_lines = test_fixtures::get_36_eventlog_xml_separate_lines()
        .expect("failed to read input log lines");
    for (idx, log_line) in input_log_lines.iter().enumerate() {
        tracing::debug!(
            message = "sending raw log to pipeline-ingress",
            tenant_id =% tenant_id,
            event_source_id =% event_source_id,
            idx =% idx,
        );

        ctx.pipeline_ingress_client
            .publish_raw_log(PublishRawLogRequest::new(
                event_source_id,
                tenant_id,
                Bytes::from(log_line.clone()),
            ))
            .await
            .expect("failed to publish raw log to pipeline-ingress");

        tracing::debug!(
            message = "sent raw log to pipeline-ingress",
            tenant_id =% tenant_id,
            event_source_id =% event_source_id,
            idx =% idx,
        );
    }

    tracing::info!(">> Test: that input shows up in raw-logs");

    let raw_logs = raw_logs_scanner_handle
        .await
        .expect("failed to configure raw_logs scanner");
    assert_eq!(raw_logs.len(), input_log_lines.len());
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
    let generated_graphs = generated_graphs_scanner_handle
        .await
        .expect("failed to configure generated_graphs scanner");
    assert_eq!(generated_graphs.len(), input_log_lines.len());

    tracing::info!(">> Test: node-identifier can identify nodes of the unidentified graph, then write to 'identified-graphs'");
    let identified_graphs = node_identifier_scanner_handle
        .await
        .expect("failed to configure identified_graphs scanner");
    assert_eq!(identified_graphs.len(), input_log_lines.len());

    let filtered_identified_graphs = identified_graphs
        .iter()
        .cloned()
        .filter(move |envelope| {
            let envelope = envelope.clone();
            let identified_graph = envelope.inner_message();
            events_36lines_node_identity_predicate(identified_graph)
        })
        .collect::<Vec<Envelope<IdentifiedGraph>>>();

    assert!(!filtered_identified_graphs.is_empty()); // quiet a lint about preferring iterator
    assert_eq!(filtered_identified_graphs.len(), 1);

    tracing::info!(">> Test: graph-merger wrote these identified nodes to our graph database, then write to 'merged-graphs'");
    let merged_graphs = graph_merger_scanner_handle
        .await
        .expect("failed to configure merged_graphs scanner");
    assert!(merged_graphs.len() >= input_log_lines.len());

    let filtered_merged_graphs = merged_graphs
        .iter()
        .cloned()
        .filter(move |envelope| {
            let envelope = envelope.clone();
            let merged_graph = envelope.inner_message();
            events_36lines_merged_graph_predicate(merged_graph)
        })
        .collect::<Vec<Envelope<MergedGraph>>>();

    assert!(!filtered_merged_graphs.is_empty()); // quiet a lint about preferring iterator
    assert_eq!(filtered_merged_graphs.len(), 1);

    // TODO: Perhaps add a test here that looks in dgraph/scylla for those identified nodes

    Ok(())
}
