#![cfg(feature = "integration_tests")]

use bytes::Bytes;
use clap::Parser;
use grapl_tracing::{
    setup_tracing,
    WorkerGuard,
};
use kafka::{
    config::ConsumerConfig,
    test_utils::topic_scanner::KafkaTopicScanner,
};
use rust_proto::{
    client_factory::{
        build_grpc_client_with_options,
        services::PipelineIngressClientConfig,
        BuildGrpcClientOptions,
    },
    graplinc::grapl::{
        api::{
            graph::v1beta1::{
                ImmutableUintProp,
                MergedGraph,
                MergedNode,
                Property,
            },
            pipeline_ingress::v1beta1::{
                client::PipelineIngressClient,
                PublishRawLogRequest,
            },
        },
        pipeline::v1beta1::Envelope,
    },
};
use test_context::{
    test_context,
    AsyncTestContext,
};
use tracing::Instrument;
use uuid::Uuid;

fn find_node<'a>(
    graph: &'a MergedGraph,
    o_p_name: &str,
    o_p_value: Property,
) -> Option<&'a MergedNode> {
    graph.nodes.values().find(|n| {
        n.properties.iter().any(|(p_name, p_value)| {
            p_name.as_str() == o_p_name && p_value.property.clone() == o_p_value
        })
    })
}

struct GraphMergerTestContext {
    pipeline_ingress_client: PipelineIngressClient,
    consumer_config: ConsumerConfig,
    _guard: WorkerGuard,
}

const CONSUMER_TOPIC: &'static str = "merged-graphs";
const SERVICE_NAME: &'static str = "graph-merger-integration-tests";

#[async_trait::async_trait]
impl AsyncTestContext for GraphMergerTestContext {
    async fn setup() -> Self {
        let _guard = setup_tracing(SERVICE_NAME).expect("setup_tracing");

        let client_config = PipelineIngressClientConfig::parse();
        let pipeline_ingress_client = build_grpc_client_with_options(
            client_config,
            BuildGrpcClientOptions {
                perform_healthcheck: true,
                ..Default::default()
            },
        )
        .await
        .expect("pipeline_ingress_client");

        let consumer_config = ConsumerConfig::with_topic(CONSUMER_TOPIC);

        GraphMergerTestContext {
            pipeline_ingress_client,
            consumer_config,
            _guard,
        }
    }
}

#[test_context(GraphMergerTestContext)]
#[tokio::test]
async fn test_sysmon_event_produces_merged_graph(
    ctx: &mut GraphMergerTestContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_source_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();

    let kafka_scanner = KafkaTopicScanner::new(ctx.consumer_config.clone())?
        .contains(move |envelope: Envelope<MergedGraph>| -> bool {
            let envelope_tenant_id = envelope.tenant_id();
            let envelope_event_source_id = envelope.event_source_id();
            let merged_graph = envelope.inner_message();
            if envelope.tenant_id() == tenant_id && envelope.event_source_id() == event_source_id {
                tracing::debug!(
                    message="found message with tenant id and event source id",
                    merged_graph=?merged_graph,
                );
                let parent_process = find_node(
                    &merged_graph,
                    "process_id",
                    ImmutableUintProp { prop: 6132 }.into(),
                )
                .expect("parent process missing");

                let child_process = find_node(
                    &merged_graph,
                    "process_id",
                    ImmutableUintProp { prop: 5752 }.into(),
                )
                .expect("child process missing");

                // NOTE: here, unlike node-identifier, we expect the edge
                // connecting the parent and child proceses to be *absent*
                // in the message emitted to the merged-graphs topic. The
                // reason for this is that downstream services (analyzers)
                // don't operate on edges, just nodes. So the view of the
                // graph diverges at the graph-merger--we now tell one story
                // in our Kafka messages and a totally different story in
                // Dgraph. This is confusing and we should fix it:
                //
                // https://app.zenhub.com/workspaces/grapl-6036cbd36bacff000ef314f2/issues/grapl-security/issue-tracker/950
                !merged_graph
                    .edges
                    .get(parent_process.get_node_key())
                    .iter()
                    .flat_map(|edge_list| edge_list.edges.iter())
                    .any(|edge| edge.to_node_key == child_process.get_node_key())
            } else {
                false
            }
        })
        .await?;

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

    tracing::info!("waiting for kafka_scanner to complete");
    kafka_scanner
        .get_listen_result()
        .instrument(tracing::debug_span!("kafka_scanner"))
        .await??;

    Ok(())
}
