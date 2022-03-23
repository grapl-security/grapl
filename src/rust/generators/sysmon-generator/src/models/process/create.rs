use endpoint_plugin::{
    AssetNode,
    FileNode,
    IAssetNode,
    IFileNode,
    IProcessNode,
    ProcessNode,
};
use rust_proto::graph_descriptions::*;
use sysmon_parser::{
    event_data::ProcessCreateEventData,
    System,
};

use crate::{
    error::SysmonGeneratorError,
    models::{
        get_image_name,
        strip_file_zone_identifier,
        utc_to_epoch,
    },
};

/// Creates a graph decribing a `ProcessCreateEvent`.
///
/// Graph generation for a `ProcessCreateEvent` includes the following:
/// * An `Asset` node - indicating the asset in which the process was created
/// * A parent `Process` node - indicating the process that created the subject process
/// * A subject `Process` node - indicating the process created per the `ProcessCreateEvent`
/// * A process `File` node - indicating the file executed in creating the new process
#[tracing::instrument]
pub fn generate_process_create_subgraph(
    system: &System,
    event_data: &ProcessCreateEventData<'_>,
) -> Result<GraphDescription, SysmonGeneratorError> {
    tracing::trace!("generating graph from event");

    let timestamp = utc_to_epoch(&event_data.utc_time)?;
    let mut graph = GraphDescription::new();

    let mut asset = AssetNode::new(AssetNode::static_strategy());
    asset
        .with_asset_id(&system.computer)
        .with_hostname(&system.computer);

    let mut parent = ProcessNode::new(ProcessNode::session_strategy());
    parent
        .with_asset_id(&system.computer)
        .with_process_id(event_data.parent_process_id)
        .with_process_name(get_image_name(&event_data.parent_image))
        .with_process_command_line(&event_data.parent_command_line)
        .with_last_seen_timestamp(timestamp);

    let mut child = ProcessNode::new(ProcessNode::session_strategy());
    child
        .with_asset_id(&system.computer)
        .with_process_name(get_image_name(&event_data.image))
        .with_process_command_line(&event_data.command_line)
        .with_process_id(event_data.process_id)
        .with_created_timestamp(timestamp);

    let mut child_exe = FileNode::new(FileNode::session_strategy());
    child_exe
        .with_asset_id(&system.computer)
        .with_last_seen_timestamp(timestamp)
        .with_file_path(strip_file_zone_identifier(&event_data.image));

    graph.add_edge(
        "process_asset",
        parent.clone_node_key(),
        asset.clone_node_key(),
    );

    graph.add_edge(
        "process_asset",
        child.clone_node_key(),
        asset.clone_node_key(),
    );

    graph.add_edge(
        "bin_file",
        child.clone_node_key(),
        child_exe.clone_node_key(),
    );

    graph.add_edge(
        "files_on_asset",
        asset.clone_node_key(),
        child_exe.clone_node_key(),
    );

    graph.add_edge("children", parent.clone_node_key(), child.clone_node_key());

    graph.add_node(asset);
    graph.add_node(parent);
    graph.add_node(child);
    graph.add_node(child_exe);

    Ok(graph)
}
#[cfg(test)]
mod tests {
    use rust_proto::graph_descriptions::{
        node_property::Property,
        ImmutableUintProp,
    };
    use sysmon_parser::EventData;

    use super::*;

    fn find_node<'a>(
        graph: &'a GraphDescription,
        o_p_name: &str,
        o_p_value: Property,
    ) -> Option<&'a NodeDescription> {
        graph.nodes.values().find(|n| {
            n.properties.iter().any(|(p_name, p_value)| {
                p_name.as_str() == o_p_name && p_value.property.clone().unwrap() == o_p_value
            })
        })
    }

    #[test]
    fn process_create() {
        // Given - A sysmon process creation event
        // When - We generate a graph from the event
        // Then - We expect a graph with a parent and child process, and an edge between them

        let event = r#"<Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'><System><Provider Name='Microsoft-Windows-Sysmon' Guid='{5770385F-C22A-43E0-BF4C-06F5698FFBD9}'/><EventID>1</EventID><Version>5</Version><Level>4</Level><Task>1</Task><Opcode>0</Opcode><Keywords>0x8000000000000000</Keywords><TimeCreated SystemTime='2019-07-24T18:05:14.402156600Z'/><EventRecordID>550</EventRecordID><Correlation/><Execution ProcessID='3324' ThreadID='3220'/><Channel>Microsoft-Windows-Sysmon/Operational</Channel><Computer>DESKTOP-FVSHABR</Computer><Security UserID='S-1-5-18'/></System><EventData><Data Name='RuleName'></Data><Data Name='UtcTime'>2019-07-24 18:05:14.399</Data><Data Name='ProcessGuid'>{87E8D3BD-9DDA-5D38-0000-0010A3941D00}</Data><Data Name='ProcessId'>5752</Data><Data Name='Image'>C:\Windows\System32\cmd.exe</Data><Data Name='FileVersion'>10.0.10240.16384 (th1.150709-1700)</Data><Data Name='Description'>Windows Command Processor</Data><Data Name='Product'>Microsoft� Windows� Operating System</Data><Data Name='Company'>Microsoft Corporation</Data><Data Name='OriginalFileName'>Cmd.Exe</Data><Data Name='CommandLine'>"cmd" /C "msiexec /quiet /i cmd.msi"</Data><Data Name='CurrentDirectory'>C:\Users\grapltest\Downloads\</Data><Data Name='User'>DESKTOP-FVSHABR\grapltest</Data><Data Name='LogonGuid'>{87E8D3BD-99C8-5D38-0000-002088140200}</Data><Data Name='LogonId'>0x21488</Data><Data Name='TerminalSessionId'>1</Data><Data Name='IntegrityLevel'>Medium</Data><Data Name='Hashes'>MD5=A6177D080759CF4A03EF837A38F62401,SHA256=79D1FFABDD7841D9043D4DDF1F93721BCD35D823614411FD4EAB5D2C16A86F35</Data><Data Name='ParentProcessGuid'>{87E8D3BD-9DD8-5D38-0000-00109F871D00}</Data><Data Name='ParentProcessId'>6132</Data><Data Name='ParentImage'>C:\Users\grapltest\Downloads\svchost.exe</Data><Data Name='ParentCommandLine'>.\svchost.exe</Data></EventData></Event>"#;
        let event = sysmon_parser::SysmonEvent::from_str(event).unwrap();

        let event_data = match event.event_data {
            EventData::ProcessCreate(event_data) => event_data,
            _ => panic!("must be ProcessCreate"),
        };

        let graph: GraphDescription = generate_process_create_subgraph(&event.system, &event_data)
            .expect("failed to generate graph");

        let process_a = find_node(
            &graph,
            "process_id",
            ImmutableUintProp { prop: 6132 }.into(),
        )
        .expect("parent process missing");

        let process_b = find_node(
            &graph,
            "process_id",
            ImmutableUintProp { prop: 5752 }.into(),
        )
        .expect("child process missing");

        let a_edges = graph.edges.get(process_a.get_node_key());
        let edge_to_b = a_edges
            .iter()
            .flat_map(|e| e.edges.iter())
            .find(|e| e.to_node_key == process_b.get_node_key());
        let edge_to_b = edge_to_b.expect("missing edge to b");
        assert_eq!(edge_to_b.edge_name, "children");
    }
}
