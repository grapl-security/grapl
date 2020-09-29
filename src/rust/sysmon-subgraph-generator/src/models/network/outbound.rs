use sysmon::NetworkEvent;

use graph_descriptions::graph_description::*;

use graph_descriptions::network_connection::NetworkConnectionState;
use graph_descriptions::node::NodeT;
use graph_descriptions::process::ProcessState;

use crate::models::utc_to_epoch;
use graph_descriptions::process_outbound_connection::ProcessOutboundConnectionState;

/// Creates a subgraph describing an outbound `NetworkEvent`
///
/// Subgraph generation for an outbound `NetworkEvent` includes the following:
/// * An `Asset` node - indicating the asset in which the outbound `NetworkEvent` occurred
/// * A `Process` node - indicating the process which triggered the outbound `NetworkEvent`
/// * A subject `OutboundConnection` node - indicating the network connection triggered by the process
/// * Source and Destination IP Address and Port nodes
/// * IP connection and Network connection nodes
pub fn generate_outbound_connection_subgraph(
    conn_log: &NetworkEvent,
) -> Result<Graph, failure::Error> {
    let timestamp = utc_to_epoch(&conn_log.event_data.utc_time)?;

    let mut graph = Graph::new(timestamp);

    let asset = AssetBuilder::default()
        .asset_id(conn_log.system.computer.computer.clone())
        .hostname(conn_log.system.computer.computer.clone())
        .build()
        .map_err(|err| failure::err_msg(err))?;

    // A process creates an outbound connection to dst_port
    let process = ProcessBuilder::default()
        .asset_id(conn_log.system.computer.computer.clone())
        .hostname(conn_log.system.computer.computer.clone())
        .state(ProcessState::Existing)
        .process_id(conn_log.event_data.process_id)
        .last_seen_timestamp(timestamp)
        .build()
        .map_err(|err| failure::err_msg(err))?;

    let outbound = ProcessOutboundConnectionBuilder::default()
        .asset_id(conn_log.system.computer.computer.clone())
        .hostname(conn_log.system.computer.computer.clone())
        .state(ProcessOutboundConnectionState::Connected)
        .ip_address(conn_log.event_data.source_ip.clone())
        .protocol(conn_log.event_data.protocol.clone())
        .port(conn_log.event_data.source_port)
        .created_timestamp(timestamp)
        .build()
        .map_err(|err| failure::err_msg(err))?;

    let src_ip = IpAddressBuilder::default()
        .ip_address(conn_log.event_data.source_ip.clone())
        .last_seen_timestamp(timestamp)
        .build()
        .map_err(|err| failure::err_msg(err))?;

    let dst_ip = IpAddressBuilder::default()
        .ip_address(conn_log.event_data.destination_ip.clone())
        .last_seen_timestamp(timestamp)
        .build()
        .map_err(|err| failure::err_msg(err))?;

    let src_port = IpPortBuilder::default()
        .ip_address(conn_log.event_data.source_ip.clone())
        .port(conn_log.event_data.source_port)
        .protocol(conn_log.event_data.protocol.clone())
        .build()
        .map_err(|err| failure::err_msg(err))?;

    let dst_port = IpPortBuilder::default()
        .ip_address(conn_log.event_data.destination_ip.clone())
        .port(conn_log.event_data.destination_port)
        .protocol(conn_log.event_data.protocol.clone())
        .build()
        .map_err(|err| failure::err_msg(err))?;

    let network_connection = NetworkConnectionBuilder::default()
        .state(NetworkConnectionState::Created)
        .src_ip_address(conn_log.event_data.source_ip.clone())
        .src_port(conn_log.event_data.source_port)
        .dst_ip_address(conn_log.event_data.destination_ip.clone())
        .dst_port(conn_log.event_data.destination_port)
        .protocol(conn_log.event_data.protocol.clone())
        .created_timestamp(timestamp)
        .build()
        .map_err(|err| failure::err_msg(err))?;

    let ip_connection = IpConnectionBuilder::default()
        .state(NetworkConnectionState::Created)
        .src_ip_address(conn_log.event_data.source_ip.clone())
        .dst_ip_address(conn_log.event_data.destination_ip.clone())
        .protocol(conn_log.event_data.protocol.clone())
        .created_timestamp(timestamp)
        .build()
        .map_err(|err| failure::err_msg(err))?;

    // An asset is assigned an IP
    graph.add_edge("asset_ip", asset.clone_node_key(), src_ip.clone_node_key());

    // A process spawns on an asset
    graph.add_edge(
        "asset_processes",
        asset.clone_node_key(),
        process.clone_node_key(),
    );

    // A process creates a connection
    graph.add_edge(
        "created_connection",
        process.clone_node_key(),
        outbound.clone_node_key(),
    );

    // The connection is over an IP + Port
    graph.add_edge(
        "connected_over",
        outbound.clone_node_key(),
        src_port.clone_node_key(),
    );

    // The outbound process connection is to a dst ip + port
    graph.add_edge(
        "external_connection",
        outbound.clone_node_key(),
        dst_port.clone_node_key(),
    );

    // There is a network connection between the src and dst ports
    graph.add_edge(
        "outbound_connection_to",
        src_port.clone_node_key(),
        network_connection.clone_node_key(),
    );

    graph.add_edge(
        "inbound_connection_to",
        network_connection.clone_node_key(),
        dst_port.clone_node_key(),
    );

    // There is also a connection between the two IP addresses

    graph.add_edge(
        "ip_connection_to",
        src_ip.clone_node_key(),
        ip_connection.clone_node_key(),
    );

    graph.add_edge(
        "ip_connection_to",
        ip_connection.clone_node_key(),
        dst_ip.clone_node_key(),
    );

    graph.add_node(asset);
    graph.add_node(process);
    graph.add_node(outbound);
    graph.add_node(src_ip);
    graph.add_node(dst_ip);
    graph.add_node(src_port);
    graph.add_node(dst_port);
    graph.add_node(network_connection);
    graph.add_node(ip_connection);

    Ok(graph)
}
