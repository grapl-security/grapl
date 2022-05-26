// use endpoint_plugin::{
//     AssetNode,
//     FileNode,
//     IAssetNode,
//     IFileNode,
//     IProcessNode,
//     ProcessNode,
// };

use rust_proto::graph_descriptions::*;
use sysmon_parser::{
    event_data::ProcessCreateEventData,
    System,
};

use crate::model::nodes::*;

/// Creates a graph decribing a `ProcessCreateEvent`.
#[tracing::instrument]
pub fn generate_process_create_subgraph(
    system: &System,
    event_data: &ProcessCreateEventData<'_>,
) -> GraphDescription {
    tracing::trace!("generating graph from ProcessCreate event");

    let timestamp = event_data.utc_time.timestamp_millis();
    let mut graph = GraphDescription::new();

    let asset = AssetNode::from(system);

    let mut new_process = ProcessNode::new(ProcessNode::static_strategy());
    new_process
        .with_pid(event_data.parent_process_id as i64)
        .with_guid(event_data.process_guid.to_string())
        .with_exe(&event_data.image);

    let mut parent = ProcessNode::new(ProcessNode::static_strategy());
    parent
        .with_pid(event_data.parent_process_id as i64)
        .with_guid(event_data.parent_process_guid.to_string())
        .with_exe(&event_data.parent_image);

    let mut process_spawn = ProcessSpawnNode::new(ProcessSpawnNode::static_strategy());
    process_spawn
        .with_timestamp(timestamp)
        .with_cmdline(&event_data.command_line)
        .with_current_directory(&event_data.current_directory)
        .with_user(&event_data.user)
        // identity-only fields
        .with_parent_guid(event_data.parent_process_guid.to_string())
        .with_child_guid(event_data.process_guid.to_string());

    // add parent_user to ProcessSpawnNode if available
    if let Some(parent_user) = &event_data.parent_user {
        process_spawn.with_parent_user(parent_user);
    }

    let mut process_exe = FileNode::new(FileNode::static_strategy());
    process_exe
        .with_asset_id(&system.computer)
        .with_path(&event_data.image);

    graph.add_edge(
        "asset_processes",
        asset.clone_node_key(),
        new_process.clone_node_key(),
    );

    graph.add_edge(
        "asset_processes",
        asset.clone_node_key(),
        parent.clone_node_key(),
    );

    graph.add_edge(
        "process_exe",
        new_process.clone_node_key(),
        process_exe.clone_node_key(),
    );

    graph.add_edge(
        "asset_files",
        asset.clone_node_key(),
        process_exe.clone_node_key(),
    );

    graph.add_edge(
        "children",
        parent.clone_node_key(),
        new_process.clone_node_key(),
    );

    graph.add_edge(
        "spawned_a",
        parent.clone_node_key(),
        process_spawn.clone_node_key(),
    );

    graph.add_edge(
        "spawned_b",
        process_spawn.clone_node_key(),
        new_process.clone_node_key(),
    );

    graph.add_node(asset);
    graph.add_node(parent);
    graph.add_node(new_process);
    graph.add_node(process_exe);
    graph.add_node(process_spawn);

    graph
}
