use graph_descriptions::graph_description::*;
use graph_descriptions::node::NodeT;
use graph_descriptions::process::ProcessState;
use serde::{Deserialize, Serialize};

use graph_descriptions::process_outbound_connection::ProcessOutboundConnectionState;

use graph_descriptions::network_connection::NetworkConnectionState;

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

impl From<ProcessOutboundConnectionLog> for Graph {
    fn from(conn_log: ProcessOutboundConnectionLog) -> Self {
        let mut graph = Graph::new(conn_log.timestamp);

        let asset = AssetBuilder::default()
            .asset_id(conn_log.src_hostname.clone())
            .hostname(conn_log.src_hostname.clone())
            .build()
            .expect("outbound_traffic.asset");

        // A process creates an outbound connection to dst_port
        let process = ProcessBuilder::default()
            .asset_id(conn_log.src_hostname.clone())
            .state(ProcessState::Existing)
            .process_id(conn_log.pid)
            .last_seen_timestamp(conn_log.timestamp)
            .build()
            .expect("outbound_traffic.process");

        let outbound = ProcessOutboundConnectionBuilder::default()
            .asset_id(conn_log.src_hostname.clone())
            .ip_address(conn_log.src_ip_addr.clone())
            .protocol(conn_log.protocol.clone())
            .state(ProcessOutboundConnectionState::Connected)
            .port(conn_log.src_port)
            .created_timestamp(conn_log.timestamp)
            .build()
            .expect("outbound_traffic.inbound");

        let src_ip = IpAddressBuilder::default()
            .ip_address(conn_log.src_ip_addr.clone())
            .last_seen_timestamp(conn_log.timestamp)
            .build()
            .expect("outbound_traffic.dst_ip");

        let dst_ip = IpAddressBuilder::default()
            .ip_address(conn_log.dst_ip_addr.clone())
            .last_seen_timestamp(conn_log.timestamp)
            .build()
            .expect("outbound_traffic.src_ip");

        let src_port = IpPortBuilder::default()
            .ip_address(conn_log.src_ip_addr.clone())
            .port(conn_log.src_port)
            .protocol(conn_log.protocol.clone())
            .build()
            .expect("outbound_traffic.src_port");

        let dst_port = IpPortBuilder::default()
            .ip_address(conn_log.dst_ip_addr.clone())
            .port(conn_log.dst_port)
            .protocol(conn_log.protocol.clone())
            .build()
            .expect("outbound_traffic.dst_port");

        let network_connection = NetworkConnectionBuilder::default()
            .state(NetworkConnectionState::Created)
            .src_ip_address(conn_log.src_ip_addr)
            .src_port(conn_log.src_port)
            .dst_ip_address(conn_log.dst_ip_addr)
            .dst_port(conn_log.dst_port)
            .protocol(conn_log.protocol)
            .created_timestamp(conn_log.timestamp)
            .build()
            .expect("outbound_traffic.network_connection");

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
            "created_connections",
            process.clone_node_key(),
            outbound.clone_node_key(),
        );

        // The connection is over an IP + Port
        graph.add_edge(
            "connected_over",
            outbound.clone_node_key(),
            src_port.clone_node_key(),
        );

        // The connection is to a dst ip + port
        graph.add_edge(
            "connected_to",
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

        graph.add_node(asset);
        graph.add_node(process);
        graph.add_node(outbound);
        graph.add_node(src_ip);
        graph.add_node(dst_ip);
        graph.add_node(src_port);
        graph.add_node(dst_port);
        graph.add_node(network_connection);

        graph
    }
}
