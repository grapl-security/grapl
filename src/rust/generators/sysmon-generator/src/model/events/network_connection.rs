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

    let timestamp = event_data.utc_time.timestamp_millis();

    let asset = AssetNode::from(system);

    let mut process = ProcessNode::new(ProcessNode::static_strategy());
    process
        .with_pid(event_data.process_id as i64)
        .with_guid(event_data.process_guid.to_string())
        .with_exe(&event_data.image);

    let mut process_exe = FileNode::new(FileNode::static_strategy());
    process_exe
        .with_asset_id(&system.computer)
        .with_path(&event_data.image);

    let mut src_network_socket_address =
        NetworkSocketAddressNode::new(NetworkSocketAddressNode::identity_strategy());
    src_network_socket_address
        .with_transport_protocol(&event_data.protocol)
        .with_port_number(event_data.source_port as i64)
        // identity-only
        .with_ip_address(event_data.source_ip.to_string());

    let mut dest_network_socket_address =
        NetworkSocketAddressNode::new(NetworkSocketAddressNode::identity_strategy());
    dest_network_socket_address
        .with_transport_protocol(&event_data.protocol)
        .with_port_number(event_data.destination_port as i64)
        // identity-only
        .with_ip_address(event_data.destination_ip.to_string());

    let mut tcp_connection = TcpConnectionNode::new(TcpConnectionNode::static_strategy());
    tcp_connection
        .with_timestamp(timestamp)
        // identity-only
        .with_src_port(event_data.source_port as i64)
        .with_dst_port(event_data.destination_port as i64)
        .with_src_ip_address(event_data.source_ip.to_string())
        .with_dst_ip_address(event_data.destination_ip.to_string())
        .with_transport_protocol(&event_data.protocol)
        .with_process_guid(event_data.process_guid.to_string());

    graph.add_edge(
        "asset_processes",
        asset.clone_node_key(),
        process.clone_node_key(),
    );

    graph.add_edge(
        "process_exe",
        process.clone_node_key(),
        process_exe.clone_node_key(),
    );

    graph.add_edge(
        "asset_files",
        asset.clone_node_key(),
        process_exe.clone_node_key(),
    );

    // process -> network sockets
    graph.add_edge(
        "process_socket_outbound",
        process.clone_node_key(),
        src_network_socket_address.clone_node_key(),
    );

    graph.add_edge(
        "process_socket_inbound",
        process.clone_node_key(),
        dest_network_socket_address.clone_node_key(),
    );

    graph.add_edge(
        "tcp_connection_to_a", 
        src_network_socket_address.clone_node_key(), 
        tcp_connection.clone_node_key()
    );

    graph.add_edge(
        "tcp_connection_to_b", 
        tcp_connection.clone_node_key(), 
        dest_network_socket_address.clone_node_key()
    );

    // handle ipv6 and ipv6
    if event_data.source_is_ipv6 {
        let mut src_ipv6_address = IpV6AddressNode::new(IpV6AddressNode::static_strategy());
        src_ipv6_address.with_address(event_data.source_ip.to_string());

        let mut dst_ipv6_address = IpV6AddressNode::new(IpV6AddressNode::static_strategy());
        dst_ipv6_address.with_address(event_data.destination_ip.to_string());

        graph.add_edge(
            "socket_ipv6_address",
            src_network_socket_address.clone_node_key(),
            src_ipv6_address.clone_node_key(),
        );

        graph.add_edge(
            "socket_ipv6_address",
            dest_network_socket_address.clone_node_key(),
            dst_ipv6_address.clone_node_key(),
        );

        graph.add_node(src_ipv6_address);
        graph.add_node(dst_ipv6_address);
    } else { // ipv4 for both src and dst
        let mut src_ipv4_address = IpV4AddressNode::new(IpV4AddressNode::static_strategy());
        src_ipv4_address.with_address(event_data.source_ip.to_string());

        let mut dst_ipv4_address = IpV4AddressNode::new(IpV4AddressNode::static_strategy());
        dst_ipv4_address.with_address(event_data.destination_ip.to_string());

        graph.add_edge(
            "socket_ipv4_address",
            src_network_socket_address.clone_node_key(),
            src_ipv4_address.clone_node_key(),
        );

        graph.add_edge(
            "socket_ipv4_address",
            dest_network_socket_address.clone_node_key(),
            dst_ipv4_address.clone_node_key(),
        );

        graph.add_node(src_ipv4_address);
        graph.add_node(dst_ipv4_address);
    }

    graph.add_node(asset);
    graph.add_node(process);
    graph.add_node(process_exe);
    graph.add_node(src_network_socket_address);
    graph.add_node(dest_network_socket_address);
    graph.add_node(tcp_connection);

    graph
}
