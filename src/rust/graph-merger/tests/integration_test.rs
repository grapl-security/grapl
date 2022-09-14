#![cfg(feature = "integration_tests")]

use std::time::Duration;

use bytes::Bytes;
use clap::Parser;
use e2e_tests::test_utils::context::{
    E2eTestContext,
    SetupResult,
};
use kafka::{
    config::ConsumerConfig,
    test_utils::topic_scanner::KafkaTopicScanner,
};
use rust_proto::{
    client_factory::{
        build_grpc_client,
        services::{
            ScyllaProvisionerClientConfig,
            UidAllocatorClientConfig,
        },
    },
    graplinc::grapl::{
        api::{
            pipeline_ingress::v1beta1::PublishRawLogRequest,
            plugin_sdk::analyzers::v1beta1::messages::{
                UInt64PropertyUpdate,
                Update,
                Updates,
            },
            scylla_provisioner::v1beta1::messages::ProvisionGraphForTenantRequest,
            uid_allocator::v1beta1::messages::CreateTenantKeyspaceRequest,
        },
        pipeline::v1beta1::Envelope,
    },
};
use test_context::test_context;
use uuid::Uuid;

const CONSUMER_TOPIC: &'static str = "merged-graphs";

#[test_context(E2eTestContext)]
#[tokio::test]
async fn test_sysmon_event_produces_merged_graph(ctx: &mut E2eTestContext) -> eyre::Result<()> {
    let test_name = "test_sysmon_event_produces_merged_graph";
    let SetupResult {
        tenant_id,
        plugin_id: _,
        event_source_id,
    } = ctx.setup_sysmon_generator(test_name).await?;

    let mut uid_allocator_client = build_grpc_client(UidAllocatorClientConfig::parse()).await?;
    uid_allocator_client
        .create_tenant_keyspace(CreateTenantKeyspaceRequest { tenant_id })
        .await?;

    let provisioner_client_config = ScyllaProvisionerClientConfig::parse();
    let mut provisioner_client = build_grpc_client(provisioner_client_config).await?;

    provisioner_client
        .provision_graph_for_tenant(ProvisionGraphForTenantRequest { tenant_id })
        .await?;

    let kafka_scanner = KafkaTopicScanner::new(
        ConsumerConfig::with_topic(CONSUMER_TOPIC),
        Duration::from_secs(60),
        Envelope::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Updates::new(),
        ),
    );

    let handle = kafka_scanner
        .scan_for_tenant(tenant_id, 1, |_: Updates| true)
        .await;

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

    tracing::info!(
        message = "sending publish_raw_log request",
        event_source_id =% event_source_id,
        tenant_id =% tenant_id,
    );

    ctx.pipeline_ingress_client
        .publish_raw_log(PublishRawLogRequest::new(
            event_source_id,
            tenant_id,
            log_event,
        ))
        .await
        .expect("received error response");

    tracing::info!("waiting for kafka_scanner to complete");

    let envelopes = handle.await?;
    assert_eq!(envelopes.len(), 1);

    let envelope = envelopes[0].clone();
    assert_eq!(envelope.tenant_id(), tenant_id);
    assert_eq!(envelope.event_source_id(), event_source_id);

    let updates = envelope.inner_message();

    // Note that updates don't contain the updated value so we can't check that
    // right now
    let matching_updates_count = updates
        .iter()
        .filter(|update| {
            matches!(update, Update::Uint64Property(UInt64PropertyUpdate {property_name, ..}) if {
                property_name.value == "process_id"
            })
        })
        .count();
    assert_eq!(matching_updates_count, 2);
    //
    // let matching_updates = updates.iter().filter(|update| {
    //     matches!(update, Update::Edge(Uint64Property {property_name, ..}) if {
    //         property_name == "process_id"
    //     })
    // }).collect::<Vec<_>>();
    // assert_eq!(matching_updates.len(), 2);

    Ok(())
}
