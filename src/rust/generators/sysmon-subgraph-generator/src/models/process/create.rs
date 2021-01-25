use endpoint_plugin::{AssetNode,
                      FileNode,
                      IAssetNode,
                      IFileNode,
                      IIpPortNode,
                      IProcessInboundConnectionNode,
                      IProcessNode,
                      IProcessOutboundConnectionNode,
                      IpPortNode,
                      ProcessInboundConnectionNode,
                      ProcessNode,
                      ProcessOutboundConnectionNode};
use grapl_graph_descriptions::graph_description::*;
use sysmon::ProcessCreateEvent;

use crate::{generator::SysmonGeneratorError,
            models::{get_image_name,
                     strip_file_zone_identifier,
                     utc_to_epoch}};

/// Creates a graph decribing a `ProcessCreateEvent`.
///
/// Graph generation for a `ProcessCreateEvent` includes the following:
/// * An `Asset` node - indicating the asset in which the process was created
/// * A parent `Process` node - indicating the process that created the subject process
/// * A subject `Process` node - indicating the process created per the `ProcessCreateEvent`
/// * A process `File` node - indicating the file executed in creating the new process
pub fn generate_process_create_subgraph(
    process_start: &ProcessCreateEvent,
) -> Result<GraphDescription, SysmonGeneratorError> {
    let timestamp = utc_to_epoch(&process_start.event_data.utc_time)?;
    let mut graph = GraphDescription::new();

    let mut asset = AssetNode::new(AssetNode::static_strategy());
    asset
        .with_asset_id(process_start.system.computer.computer.clone())
        .with_hostname(process_start.system.computer.computer.clone());

    let mut parent = ProcessNode::new(ProcessNode::session_strategy());
    parent
        .with_asset_id(process_start.system.computer.computer.clone())
        .with_process_id(process_start.event_data.parent_process_id)
        .with_process_name(get_image_name(&process_start.event_data.parent_image.clone()).unwrap())
        .with_process_command_line(&process_start.event_data.parent_command_line.command_line)
        .with_last_seen_timestamp(timestamp);

    let mut child = ProcessNode::new(ProcessNode::session_strategy());
    child
        .with_asset_id(process_start.system.computer.computer.clone())
        .with_process_name(get_image_name(&process_start.event_data.image.clone()).unwrap())
        .with_process_command_line(&process_start.event_data.command_line.command_line)
        .with_process_id(process_start.event_data.process_id)
        .with_created_timestamp(timestamp);

    let mut child_exe = FileNode::new(FileNode::session_strategy());
    child_exe
        .with_asset_id(process_start.system.computer.computer.clone())
        .with_last_seen_timestamp(timestamp)
        .with_file_path(strip_file_zone_identifier(&process_start.event_data.image));

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
    graph.add_edge("parent", child.clone_node_key(), parent.clone_node_key());

    graph.add_node(asset);
    graph.add_node(parent);
    graph.add_node(child);
    graph.add_node(child_exe);

    Ok(graph)
}
