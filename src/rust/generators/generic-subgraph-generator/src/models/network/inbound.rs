use std::convert::TryFrom;

use grapl_graph_descriptions::{graph_description::*,
                               network_connection::NetworkConnectionState,
                               node::NodeT,
                               process::ProcessState,
                               process_inbound_connection::ProcessInboundConnectionState};
use serde::{Deserialize,
            Serialize};

// In an inbound connection "src" is where the connection is coming from
#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct ProcessInboundConnectionLog {
    /// The pid of the process receiving the connection
    pid: u64,
    src_ip_addr: String,
    src_port: u32,
    dst_port: u32,
    dst_hostname: String,
    dst_ip_addr: String,
    protocol: String,
    timestamp: u64,
}

impl TryFrom<ProcessInboundConnectionLog> for Graph {
    type Error = String;

    fn try_from(conn_log: ProcessInboundConnectionLog) -> Result<Self, Self::Error> {
        let mut graph = Graph::new(conn_log.timestamp);

        let asset = AssetBuilder::default()
            .asset_id(conn_log.dst_hostname.clone())
            .hostname(conn_log.dst_hostname.clone())
            .build()?;

        // A process creates an outbound connection to dst_port
        let process = ProcessBuilder::default()
            .asset_id(conn_log.dst_hostname.clone())
            .state(ProcessState::Existing)
            .process_id(conn_log.pid)
            .last_seen_timestamp(conn_log.timestamp)
            .build()?;

        let inbound = ProcessInboundConnectionBuilder::default()
            .asset_id(conn_log.dst_hostname.clone())
            .state(ProcessInboundConnectionState::Existing)
            .ip_address(conn_log.dst_ip_addr.clone())
            .protocol(conn_log.protocol.clone())
            .port(conn_log.dst_port)
            .created_timestamp(conn_log.timestamp)
            .build()?;

        let dst_ip = IpAddressBuilder::default()
            .ip_address(conn_log.dst_ip_addr.clone())
            .last_seen_timestamp(conn_log.timestamp)
            .build()?;

        let src_ip = IpAddressBuilder::default()
            .ip_address(conn_log.src_ip_addr.clone())
            .last_seen_timestamp(conn_log.timestamp)
            .build()?;

        let src_port = IpPortBuilder::default()
            .ip_address(conn_log.src_ip_addr.clone())
            .protocol(conn_log.protocol.clone())
            .port(conn_log.src_port)
            .build()?;

        let dst_port = IpPortBuilder::default()
            .ip_address(conn_log.dst_ip_addr.clone())
            .protocol(conn_log.protocol.clone())
            .port(conn_log.dst_port)
            .build()?;

        let network_connection = NetworkConnectionBuilder::default()
            .state(NetworkConnectionState::Created)
            .src_ip_address(conn_log.src_ip_addr)
            .src_port(conn_log.src_port)
            .dst_ip_address(conn_log.dst_ip_addr)
            .dst_port(conn_log.dst_port)
            .protocol(conn_log.protocol)
            .created_timestamp(conn_log.timestamp)
            .build()?;

        // An asset is assigned an IP
        graph.add_edge("asset_ip", asset.node_key.clone(), dst_ip.node_key.clone());

        // A process spawns on an asset
        graph.add_edge(
            "asset_processes",
            asset.node_key.clone(),
            process.node_key.clone(),
        );

        // A process creates a connection
        graph.add_edge(
            "received_connection",
            process.node_key.clone(),
            inbound.node_key.clone(),
        );

        // The connection is over an IP + Port
        graph.add_edge(
            "bound_port",
            inbound.node_key.clone(),
            src_port.node_key.clone(),
        );

        // The connection is to a dst ip + port
        graph.add_edge(
            "connected_to",
            inbound.node_key.clone(),
            dst_port.node_key.clone(),
        );

        // There is a network connection between the src and dst ports
        graph.add_edge(
            "outbound_connection_to",
            src_port.node_key.clone(),
            network_connection.node_key.clone(),
        );

        graph.add_edge(
            "inbound_connection_to",
            network_connection.node_key.clone(),
            dst_port.node_key.clone(),
        );

        graph.add_node(asset);
        graph.add_node(process);
        graph.add_node(inbound);
        graph.add_node(dst_ip);
        graph.add_node(src_ip);
        graph.add_node(src_port);
        graph.add_node(dst_port);
        graph.add_node(network_connection);

        Ok(graph)
    }
}
