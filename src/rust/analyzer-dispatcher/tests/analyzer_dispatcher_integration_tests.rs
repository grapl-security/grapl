#![cfg(feature = "integration_tests")]

use std::time::Duration;

use bytes::Bytes;
use integration_test_utils::{
    context::E2eTestContext,
    plugin_health::assert_eventual_health,
    test_fixtures,
};
use plugin_work_queue::test_utils::scan_analyzer_messages;
use rust_proto::graplinc::grapl::api::{
    pipeline_ingress::v1beta1::PublishRawLogRequest,
    plugin_registry::v1beta1::PluginHealthStatus,
};
use test_context::test_context;

#[test_context(E2eTestContext)]
#[tokio::test]
async fn test_analyzer_dispatcher_inserts_job_into_plugin_work_queue(
    ctx: &mut E2eTestContext,
) -> eyre::Result<()> {
    let tenant_id = ctx.create_tenant().await?;

    let event_source_id = ctx
        .create_event_source(
            tenant_id,
            "analyzer-dispatcher-test-event-source".to_string(),
            "analyzer dispatcher test event source".to_string(),
        )
        .await?;

    let generator_plugin_id = ctx
        .create_generator(
            tenant_id,
            "analyzer dispatcher test sysmon generator".to_string(),
            event_source_id,
            test_fixtures::get_sysmon_generator()?,
        )
        .await?;

    ctx.deploy_generator(generator_plugin_id).await?;

    let plugin_healthy_timeout = Duration::from_secs(60);
    assert_eventual_health(
        &ctx.plugin_registry_client,
        generator_plugin_id,
        PluginHealthStatus::Running,
        plugin_healthy_timeout,
    )
    .await?;

    let analyzer_id = ctx
        .create_analyzer(
            tenant_id,
            "analyzer dispatcher test analyzer".to_string(),
            Bytes::from("fake"), // FIXME: use a real analyzer
        )
        .await?;

    ctx.deploy_analyzer(analyzer_id).await?;

    // Send in the Raw Log Event
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
        .publish_raw_log(PublishRawLogRequest::new(
            event_source_id,
            tenant_id,
            log_event,
        ))
        .await?;

    scan_analyzer_messages(
        ctx.plugin_work_queue_psql_client.clone(),
        Duration::from_secs(30),
        analyzer_id,
    )
    .await?;

    Ok(())
}
