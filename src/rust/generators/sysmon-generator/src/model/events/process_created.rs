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

    let mut machine = MachineNode::new(MachineNode::static_strategy());
    machine
        .with_machine_id(&system.computer)
        .with_hostname(&system.computer);

    let mut process = ProcessNode::new(ProcessNode::static_strategy());
    process
        .with_pid(event_data.parent_process_id as i64)
        .with_guid(event_data.process_guid.to_string())
        .with_created_timestamp(timestamp)
        .with_cmdline(&event_data.command_line)
        .with_image(&event_data.image)
        .with_current_directory(&event_data.current_directory)
        .with_user(&event_data.user);

    let mut parent = ProcessNode::new(ProcessNode::static_strategy());
    parent
        .with_pid(event_data.parent_process_id as i64)
        .with_guid(event_data.parent_process_guid.to_string())
        .with_cmdline(&event_data.parent_command_line)
        .with_image(&event_data.parent_image);

    let mut process_image = FileNode::new(FileNode::static_strategy());
    process_image
        .with_machine_id(&system.computer)
        .with_path(&event_data.image);

    graph.add_edge(
        "machine_process",
        machine.clone_node_key(),
        process.clone_node_key(),
    );

    graph.add_edge(
        "machine_process",
        machine.clone_node_key(),
        parent.clone_node_key(),
    );

    graph.add_edge(
        "process_image",
        process.clone_node_key(),
        process_image.clone_node_key(),
    );

    graph.add_edge(
        "machine_files",
        machine.clone_node_key(),
        process_image.clone_node_key(),
    );

    graph.add_edge(
        "children",
        parent.clone_node_key(),
        process.clone_node_key(),
    );

    graph.add_node(machine);
    graph.add_node(parent);
    graph.add_node(process);
    graph.add_node(process_image);

    graph
}
