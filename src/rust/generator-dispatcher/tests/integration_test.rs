#![cfg(feature = "new_integration_tests")]

use std::time::Duration;

use bytes::Bytes;
use futures::StreamExt;
use kafka::{
    Consumer,
    ConsumerError,
};
use rust_proto_new::graplinc::grapl::{
    api::{
        graph::v1beta1::{
            GraphDescription,
            ImmutableUintProp,
        },
        pipeline_ingress::v1beta1::PublishRawLogRequest,
    },
    pipeline::v1beta2::Envelope,
};
use test_context::test_context;
use tokio::sync::oneshot;
use tracing::Instrument;
use uuid::Uuid;
mod test_utils;
use test_utils::context::GeneratorDispatcherTestContext;

#[test_context(GeneratorDispatcherTestContext)]
#[tokio::test]
async fn test_sysmon_event_produces_expected_graph(ctx: &mut GeneratorDispatcherTestContext) {
    let event_source_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();

    tracing::info!("configuring kafka consumer");
    let kafka_consumer =
        Consumer::new(ctx.consumer_config)
            .expect("could not configure kafka consumer");

    // we'll use this channel to communicate that the consumer is ready to
    // consume messages
    let (tx, rx) = oneshot::channel::<()>();

    tracing::info!("creating kafka subscriber thread");
    let kafka_subscriber = tokio::task::spawn(async move {
        let stream = kafka_consumer
            .stream()
            .expect("could not subscribe to the topic");

        // notify the consumer that we're ready to receive messages
        tx.send(())
            .expect("failed to notify sender that consumer is consuming");

        let contains_expected = stream.any(
            |res: Result<Envelope<GraphDescription>, ConsumerError>| async move {
                let envelope = res.expect("error consuming message from kafka");
                let metadata = envelope.metadata;
                let generated_graph = envelope.inner_message;

                tracing::debug!(message = "consumed kafka message");

                if metadata.tenant_id == tenant_id && metadata.event_source_id == event_source_id {
                    let parent_process = find_node(
                        &generated_graph,
                        "process_id",
                        ImmutableUintProp { prop: 6132 }.into(),
                    )
                    .expect("parent process missing");

                    let child_process = find_node(
                        &generated_graph,
                        "process_id",
                        ImmutableUintProp { prop: 5752 }.into(),
                    )
                    .expect("child process missing");

                    let parent_to_child_edge = generated_graph
                        .edges
                        .get(parent_process.get_node_key())
                        .iter()
                        .flat_map(|edge_list| edge_list.edges.iter())
                        .find(|edge| edge.to_node_key == child_process.get_node_key())
                        .expect("missing edge from parent to child");

                    parent_to_child_edge.edge_name == "children"
                } else {
                    false
                }
            },
        );

        tracing::info!("consuming kafka messages for 30s");
        assert!(
            tokio::time::timeout(Duration::from_millis(30000), contains_expected)
                .await
                .expect("failed to consume expected message within 30s")
        );
    });

    // wait for the kafka consumer to start consuming
    tracing::info!("waiting for kafka consumer to report ready");
    rx.await
        .expect("failed to receive notification that consumer is consuming");

    let log_event: Bytes = r#"i am some sort of raw log event"#;

    tracing::info!("sending publish_raw_log request");
    ctx.pipeline_ingress_client
        .publish_raw_log(PublishRawLogRequest {
            event_source_id,
            tenant_id,
            log_event,
        })
        .await
        .expect("received error response");

    tracing::info!("waiting for kafka_subscriber to complete");
    kafka_subscriber
        .instrument(tracing::debug_span!("kafka_subscriber"))
        .await
        .expect("could not join kafka subscriber");
}
