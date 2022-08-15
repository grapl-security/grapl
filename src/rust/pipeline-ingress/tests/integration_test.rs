#![cfg(feature = "integration_tests")]

use std::time::Duration;

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
        build_grpc_client,
        services::PipelineIngressClientConfig,
    },
    graplinc::grapl::{
        api::pipeline_ingress::v1beta1::{
            client::PipelineIngressClient,
            PublishRawLogRequest,
        },
        pipeline::v1beta1::{
            Envelope,
            RawLog,
        },
    },
};
use test_context::{
    test_context,
    AsyncTestContext,
};
use uuid::Uuid;

static CONSUMER_TOPIC: &'static str = "raw-logs";

struct PipelineIngressTestContext {
    grpc_client: PipelineIngressClient,
    _guard: WorkerGuard,
}

#[async_trait::async_trait]
impl AsyncTestContext for PipelineIngressTestContext {
    async fn setup() -> Self {
        let _guard = setup_tracing("pipeline-ingress-integration-tests").expect("setup_tracing");

        let client_config = PipelineIngressClientConfig::parse();
        let pipeline_ingress_client = build_grpc_client(client_config)
            .await
            .expect("pipeline_ingress_client");

        PipelineIngressTestContext {
            grpc_client: pipeline_ingress_client,
            _guard,
        }
    }
}

#[tracing::instrument(skip(ctx))]
#[test_context(PipelineIngressTestContext)]
#[tokio::test]
async fn test_publish_raw_log_sends_message_to_kafka(
    ctx: &mut PipelineIngressTestContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_source_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
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

    let kafka_scanner = KafkaTopicScanner::new(
        ConsumerConfig::with_topic(CONSUMER_TOPIC),
        Duration::from_secs(30),
        Envelope::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            RawLog::new(log_event.clone()),
        ),
    );

    let handle = kafka_scanner
        .scan_for_tenant(tenant_id, 1, |_: RawLog| true)
        .await;

    tracing::info!(
        message = "sending publish_raw_log request",
        tenant_id =% tenant_id,
        event_source_id =% event_source_id,
    );

    ctx.grpc_client
        .publish_raw_log(PublishRawLogRequest::new(
            event_source_id,
            tenant_id,
            log_event.clone(),
        ))
        .await
        .expect("received error response");

    tracing::info!("waiting for kafka_scanner to complete");
    let envelopes = handle.await?;

    assert_eq!(envelopes.len(), 1);

    let envelope = envelopes[0].clone();

    assert_eq!(envelope.tenant_id(), tenant_id);
    assert_eq!(envelope.event_source_id(), event_source_id);

    let raw_log = envelope.inner_message();
    assert_eq!(raw_log.log_event(), log_event);

    Ok(())
}
