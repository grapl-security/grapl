#![cfg(feature = "integration_tests")]

use std::time::Duration;

use bytes::Bytes;
use futures::StreamExt;
use grapl_tracing::{
    setup_tracing,
    WorkerGuard,
};
use kafka::{
    config::ConsumerConfig,
    Consumer,
    ConsumerError,
};
use rust_proto::{
    graplinc::grapl::{
        api::{
            graph::v1beta1::{
                IdentifiedGraph,
                IdentifiedNode,
                ImmutableUintProp,
                Property,
            },
            pipeline_ingress::v1beta1::{
                client::PipelineIngressClient,
                PublishRawLogRequest,
            },
        },
        pipeline::v1beta2::Envelope,
    },
    protocol::{
        healthcheck::client::HealthcheckClient,
        service_client::NamedService,
    },
};
use test_context::{
    test_context,
    AsyncTestContext,
};
use tokio::sync::oneshot;
use tracing::Instrument;
use uuid::Uuid;

static CONSUMER_TOPIC: &'static str = "identified-graphs";

fn find_node<'a>(
    graph: &'a IdentifiedGraph,
    o_p_name: &str,
    o_p_value: Property,
) -> Option<&'a IdentifiedNode> {
    graph.nodes.values().find(|n| {
        n.properties.iter().any(|(p_name, p_value)| {
            p_name.as_str() == o_p_name && p_value.property.clone() == o_p_value
        })
    })
}

struct NodeIdentifierTestContext {
    pipeline_ingress_client: PipelineIngressClient,
    consumer_config: ConsumerConfig,
    _guard: WorkerGuard,
}

const SERVICE_NAME: &'static str = "node-identifier-integration-tests";

#[async_trait::async_trait]
impl AsyncTestContext for NodeIdentifierTestContext {
    async fn setup() -> Self {
        let _guard = setup_tracing(SERVICE_NAME).expect("setup_tracing");

        let endpoint = std::env::var("PIPELINE_INGRESS_CLIENT_ADDRESS")
            .expect("missing environment variable PIPELINE_INGRESS_CLIENT_ADDRESS");

        tracing::info!(
            message = "waiting 10s for pipeline-ingress to report healthy",
            endpoint = %endpoint,
        );

        HealthcheckClient::wait_until_healthy(
            endpoint.clone(),
            PipelineIngressClient::SERVICE_NAME,
            Duration::from_millis(10000),
            Duration::from_millis(500),
        )
        .await
        .expect("pipeline-ingress never reported healthy");

        tracing::info!("connecting pipeline-ingress gRPC client");
        let pipeline_ingress_client = PipelineIngressClient::connect(endpoint.clone())
            .await
            .expect("could not configure gRPC client");

        let consumer_config = ConsumerConfig::with_topic(CONSUMER_TOPIC);

        NodeIdentifierTestContext {
            pipeline_ingress_client,
            consumer_config,
            _guard,
        }
    }
}

#[test_context(NodeIdentifierTestContext)]
#[tokio::test]
async fn test_sysmon_event_produces_identified_graph(ctx: &mut NodeIdentifierTestContext) {
    let event_source_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();

    tracing::info!("configuring kafka consumer");
    let kafka_consumer =
        Consumer::new(ctx.consumer_config.clone()).expect("could not configure kafka consumer");

    // we'll use this channel to communicate that the consumer is ready to
    // consume messages
    let (tx, rx) = oneshot::channel::<()>();

    tracing::info!("creating kafka subscriber thread");
    let kafka_subscriber = tokio::task::spawn(async move {
        let stream = kafka_consumer.stream();

        // notify the consumer that we're ready to receive messages
        tx.send(())
            .expect("failed to notify sender that consumer is consuming");

        let contains_expected = stream.any(
            |res: Result<Envelope<IdentifiedGraph>, ConsumerError>| async move {
                let envelope = res.expect("error consuming message from kafka");
                let metadata = envelope.metadata;
                let identified_graph = envelope.inner_message;

                tracing::debug!(message = "consumed kafka message");

                if metadata.tenant_id == tenant_id && metadata.event_source_id == event_source_id {
                    let parent_process = find_node(
                        &identified_graph,
                        "process_id",
                        ImmutableUintProp { prop: 6132 }.into(),
                    )
                    .expect("parent process missing");

                    let child_process = find_node(
                        &identified_graph,
                        "process_id",
                        ImmutableUintProp { prop: 5752 }.into(),
                    )
                    .expect("child process missing");

                    let parent_to_child_edge = identified_graph
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

    let log_event: Bytes = r#"
<Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
  <System>
    <Provider Name="Microsoft-Windows-Sysmon" Guid="{5770385F-C22A-43E0-BF4C-06F5698FFBD9}"/>
    <EventID>1</EventID>
    <Version>5</Version>
    <Level>4</Level>
    <Task>1</Task>
    <Opcode>0</Opcode>
    <Keywords>0x8000000000000000</Keywords>
    <TimeCreated SystemTime="2019-07-24T18:05:14.402156600Z"/>
    <EventRecordID>550</EventRecordID>
    <Correlation/>
    <Execution ProcessID="3324" ThreadID="3220"/>
    <Channel>Microsoft-Windows-Sysmon/Operational</Channel>
    <Computer>DESKTOP-FVSHABR</Computer>
    <Security UserID="S-1-5-18"/>
  </System>
  <EventData>
    <Data Name="RuleName"/>
    <Data Name="UtcTime">2019-07-24 18:05:14.399</Data>
    <Data Name="ProcessGuid">{87E8D3BD-9DDA-5D38-0000-0010A3941D00}</Data>
    <Data Name="ProcessId">5752</Data>
    <Data Name="Image">C:\Windows\System32\cmd.exe</Data>
    <Data Name="FileVersion">10.0.10240.16384 (th1.150709-1700)</Data>
    <Data Name="Description">Windows Command Processor</Data>
    <Data Name="Product">Microsoft&#xFFFD; Windows&#xFFFD; Operating System</Data>
    <Data Name="Company">Microsoft Corporation</Data>
    <Data Name="OriginalFileName">Cmd.Exe</Data>
    <Data Name="CommandLine">"cmd" /C "msiexec /quiet /i cmd.msi"</Data>
    <Data Name="CurrentDirectory">C:\Users\grapltest\Downloads\</Data>
    <Data Name="User">DESKTOP-FVSHABR\grapltest</Data>
    <Data Name="LogonGuid">{87E8D3BD-99C8-5D38-0000-002088140200}</Data>
    <Data Name="LogonId">0x21488</Data>
    <Data Name="TerminalSessionId">1</Data>
    <Data Name="IntegrityLevel">Medium</Data>
    <Data Name="Hashes">MD5=A6177D080759CF4A03EF837A38F62401,SHA256=79D1FFABDD7841D9043D4DDF1F93721BCD35D823614411FD4EAB5D2C16A86F35</Data>
    <Data Name="ParentProcessGuid">{87E8D3BD-9DD8-5D38-0000-00109F871D00}</Data>
    <Data Name="ParentProcessId">6132</Data>
    <Data Name="ParentImage">C:\Users\grapltest\Downloads\svchost.exe</Data>
    <Data Name="ParentCommandLine">.\svchost.exe</Data>
  </EventData>
</Event>
"#.into();

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
