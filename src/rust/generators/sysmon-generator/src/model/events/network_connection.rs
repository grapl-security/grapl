use rust_proto::graph_descriptions::*;
use sysmon_parser::{
    event_data::NetworkConnectionEventData,
    System,
};

use crate::model::nodes::*;

/// Creates a graph decribing a `NetworkConnectionEvent`.
#[tracing::instrument]
pub fn generate_network_connection_subgraph(
    system: &System,
    event_data: &NetworkConnectionEventData<'_>,
) -> GraphDescription {
    tracing::trace!("generating graph from NetworkConnection event");

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

    let mut process_image = FileNode::new(FileNode::static_strategy());
    process_image
        .with_machine_id(&system.computer)
        .with_path(&event_data.image);

    let mut dest_network_socket_address =
        NetworkSocketAddressNode::new(NetworkSocketAddressNode::identity_strategy());
    dest_network_socket_address
        .with_transport_protocol(&event_data.protocol)
        .with_ip_address(event_data.destination_ip.to_string())
        .with_port_number(event_data.destination_port as i64);

    let mut src_network_socket_address =
        NetworkSocketAddressNode::new(NetworkSocketAddressNode::identity_strategy());
    src_network_socket_address
        .with_transport_protocol(&event_data.protocol)
        .with_ip_address(event_data.source_ip.to_string())
        .with_port_number(event_data.source_port as i64);

    graph.add_edge(
        "machine_process",
        machine.clone_node_key(),
        process.clone_node_key(),
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

    // process -> network sockets
    graph.add_edge(
        "process_connected_to",
        process.clone_node_key(),
        dest_network_socket_address.clone_node_key(),
    );

    graph.add_edge(
        "process_connected_via",
        process.clone_node_key(),
        src_network_socket_address.clone_node_key(),
    );

    graph.add_edge(
        "network_connection_to",
        src_network_socket_address.clone_node_key(),
        dest_network_socket_address.clone_node_key(),
    );

    graph.add_node(machine);
    graph.add_node(src_network_socket_address);
    graph.add_node(dest_network_socket_address);
    graph.add_node(process);
    graph.add_node(process_image);

    graph
}
