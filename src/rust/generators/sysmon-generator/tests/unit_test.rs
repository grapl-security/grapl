mod generator_test_ctx;

use bytes::Bytes;
use generator_test_ctx::GeneratorTestContext;
use rust_proto::graplinc::grapl::api::{
    graph::v1beta1::{
        GraphDescription,
        ImmutableUintProp,
        NodeDescription,
        Property,
    },
    plugin_sdk::generators::v1beta1::RunGeneratorRequest,
};
use test_context::test_context;

fn find_node<'a>(
    graph: &'a GraphDescription,
    o_p_name: &str,
    o_p_value: Property,
) -> Option<&'a NodeDescription> {
    graph.nodes.values().find(|n| {
        n.properties.iter().any(|(p_name, p_value)| {
            p_name.as_str() == o_p_name && p_value.property.clone() == o_p_value
        })
    })
}

fn log_bytes() -> Bytes {
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
    log_event
}

#[test_context(GeneratorTestContext)]
#[tokio::test]
async fn test_sysmon_event_produces_expected_graph(
    ctx: &mut SysmonGeneratorTestContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = ctx
        .client
        .run_generator(RunGeneratorRequest { data: log_bytes() })
        .await?;
    let generated_graph = result.generated_graph.graph_description;

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

    assert_eq!(parent_to_child_edge.edge_name, "children");
    Ok(())
}
