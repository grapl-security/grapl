use crate::models::{get_image_name, strip_file_zone_identifier, utc_to_epoch};
use grapl_graph_descriptions::file::FileState;
use grapl_graph_descriptions::graph_description::*;
use grapl_graph_descriptions::node::NodeT;
use grapl_graph_descriptions::process::ProcessState;
use sysmon::ProcessCreateEvent;

/// Creates a subgraph describing a `ProcessCreateEvent`.
///
/// Subgraph generation for a `ProcessCreateEvent` includes the following:
/// * An `Asset` node - indicating the asset in which the process was created
/// * A parent `Process` node - indicating the process that created the subject process
/// * A subject `Process` node - indicating the process created per the `ProcessCreateEvent`
/// * A process `File` node - indicating the file executed in creating the new process
pub fn generate_process_create_subgraph(
    process_start: &ProcessCreateEvent,
) -> Result<Graph, failure::Error> {
    let timestamp = utc_to_epoch(&process_start.event_data.utc_time)?;
    let mut graph = Graph::new(timestamp);

    let asset = AssetBuilder::default()
        .asset_id(process_start.system.computer.computer.clone())
        .hostname(process_start.system.computer.computer.clone())
        .build()
        .map_err(|err| failure::err_msg(err))?;

    let parent = ProcessBuilder::default()
        .asset_id(process_start.system.computer.computer.clone())
        .state(ProcessState::Existing)
        .process_id(process_start.event_data.parent_process_id)
        .process_name(get_image_name(&process_start.event_data.parent_image.clone()).unwrap())
        .process_command_line(&process_start.event_data.parent_command_line.command_line)
        .last_seen_timestamp(timestamp)
        //        .created_timestamp(process_start.event_data.parent_process_guid.get_creation_timestamp())
        .build()
        .map_err(|err| failure::err_msg(err))?;

    let child = ProcessBuilder::default()
        .asset_id(process_start.system.computer.computer.clone())
        .process_name(get_image_name(&process_start.event_data.image.clone()).unwrap())
        .process_command_line(&process_start.event_data.command_line.command_line)
        .state(ProcessState::Created)
        .process_id(process_start.event_data.process_id)
        .created_timestamp(timestamp)
        .build()
        .map_err(|err| failure::err_msg(err))?;

    let child_exe = FileBuilder::default()
        .asset_id(process_start.system.computer.computer.clone())
        .state(FileState::Existing)
        .last_seen_timestamp(timestamp)
        .file_path(strip_file_zone_identifier(&process_start.event_data.image))
        .build()
        .map_err(|err| failure::err_msg(err))?;

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
        child_exe.clone_node_key()
    );

    graph.add_edge("children", parent.clone_node_key(), child.clone_node_key());
    graph.add_edge("parent", child.clone_node_key(), parent.clone_node_key());

    graph.add_node(asset);
    graph.add_node(parent);
    graph.add_node(child);
    graph.add_node(child_exe);

    Ok(graph)
}
