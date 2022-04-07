use rust_proto::graph_descriptions::*;
use sysmon_parser::{
    event_data::FileCreateEventData,
    System,
};

use crate::model::nodes::*;

#[tracing::instrument]
pub fn generate_file_create_subgraph(
    system: &System,
    event_data: &FileCreateEventData<'_>,
) -> GraphDescription {
    tracing::trace!("generating graph from FileCreate event");

    let mut graph = GraphDescription::new();

    let mut machine = MachineNode::new(MachineNode::static_strategy());
    machine
        .with_machine_id(&system.computer)
        .with_hostname(&system.computer);

    let mut process = ProcessNode::new(ProcessNode::static_strategy());
    process
        .with_pid(event_data.process_id as i64)
        .with_guid(event_data.process_guid.to_string())
        .with_image(&event_data.image);

    let mut file = FileNode::new(FileNode::static_strategy());
    file.with_machine_id(&system.computer)
        .with_path(&event_data.target_filename);

    graph.add_edge(
        "machine_process",
        machine.clone_node_key(),
        process.clone_node_key(),
    );

    graph.add_edge(
        "created_files",
        process.clone_node_key(),
        file.clone_node_key(),
    );

    graph.add_edge(
        "machine_files",
        machine.clone_node_key(),
        file.clone_node_key(),
    );

    graph.add_node(machine);
    graph.add_node(process);
    graph.add_node(file);

    graph
}
