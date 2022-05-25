use derive_dynamic_node::{
    GraplStaticId,
    NodeDescription,
};
use rust_proto::graph_descriptions::*;

#[derive(Debug, Clone, PartialEq, Hash, NodeDescription, GraplStaticId)]
struct TcpConnection {
    #[grapl(static_id, immutable)]
    timestamp: i64,

    // identity-only fields
    #[grapl(static_id, immutable)]
    src_port: i64,
    #[grapl(static_id, immutable)]
    dst_port: i64,
    #[grapl(static_id, immutable)]
    src_ip_address: String,
    #[grapl(static_id, immutable)]
    dst_ip_address: String,
    #[grapl(static_id, immutable)]
    transport_protocol: String,
    #[grapl(static_id, immutable)]
    process_guid: String,
}

impl ITcpConnectionNode for TcpConnectionNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}
