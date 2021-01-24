use std::convert::TryFrom;

use grapl_graph_descriptions::{graph_description::*,
                               network_connection::NetworkConnectionState,
                               node::NodeT,
                               process::ProcessState,
                               process_outbound_connection::ProcessOutboundConnectionState};
use serde::{Deserialize,
            Serialize};

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct ProcessOutboundConnectionLog {
    pid: u64,
    protocol: String,
    src_port: u32,
    dst_port: u32,
    src_hostname: String,
    src_ip_addr: String,
    dst_ip_addr: String,
    timestamp: u64,
}

impl TryFrom<ProcessOutboundConnectionLog> for Graph {
    type Error = String;

    fn try_from(conn_log: ProcessOutboundConnectionLog) -> Result<Self, Self::Error> {
        let mut graph = Graph::new(conn_log.timestamp);

        let asset = AssetBuilder::default()
            .asset_id(conn_log.src_hostname.clone())
            .hostname(conn_log.src_hostname.clone())
            .build()?;

        // A process creates an outbound connection to dst_port
        let process = ProcessBuilder::default()
            .asset_id(conn_log.src_hostname.clone())
            .state(ProcessState::Existing)
            .process_id(conn_log.pid)
            .last_seen_timestamp(conn_log.timestamp)
            .build()?;

        let outbound = ProcessOutboundConnectionBuilder::default()
            .asset_id(conn_log.src_hostname.clone())
            .ip_address(conn_log.src_ip_addr.clone())
            .protocol(conn_log.protocol.clone())
            .state(ProcessOutboundConnectionState::Connected)
            .port(conn_log.src_port)
            .created_timestamp(conn_log.timestamp)
            .build()?;

        let src_ip = IpAddressBuilder::default()
            .ip_address(conn_log.src_ip_addr.clone())
            .last_seen_timestamp(conn_log.timestamp)
            .build()?;

        let dst_ip = IpAddressBuilder::default()
            .ip_address(conn_log.dst_ip_addr.clone())
            .last_seen_timestamp(conn_log.timestamp)
            .build()?;

        let src_port = IpPortBuilder::default()
            .ip_address(conn_log.src_ip_addr.clone())
            .port(conn_log.src_port)
            .protocol(conn_log.protocol.clone())
            .build()?;

        let dst_port = IpPortBuilder::default()
            .ip_address(conn_log.dst_ip_addr.clone())
            .port(conn_log.dst_port)
            .protocol(conn_log.protocol.clone())
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
        graph.add_edge("asset_ip", asset.node_key.clone(), src_ip.node_key.clone());

        // A process spawns on an asset
        graph.add_edge(
            "asset_processes",
            asset.node_key.clone(),
            process.node_key.clone(),
        );

        // A process creates a connection
        graph.add_edge(
            "created_connections",
            process.node_key.clone(),
            outbound.node_key.clone(),
        );

        // The connection is over an IP + Port
        graph.add_edge(
            "connected_over",
            outbound.node_key.clone(),
            src_port.node_key.clone(),
        );

        // The connection is to a dst ip + port
        graph.add_edge(
            "connected_to",
            outbound.node_key.clone(),
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
        graph.add_node(outbound);
        graph.add_node(src_ip);
        graph.add_node(dst_ip);
        graph.add_node(src_port);
        graph.add_node(dst_port);
        graph.add_node(network_connection);

        Ok(graph)
    }
}
