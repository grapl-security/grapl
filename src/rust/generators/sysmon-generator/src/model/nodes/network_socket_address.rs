use derive_dynamic_node::{
    GraplStaticId,
    NodeDescription,
};
use rust_proto::graph_descriptions::*;

#[derive(Debug, Clone, PartialEq, Hash, NodeDescription, GraplStaticId)]
struct NetworkSocketAddress {
    #[grapl(static_id, immutable)]
    transport_protocol: String,
    #[grapl(static_id, immutable)]
    ip_address: String,
    #[grapl(static_id, immutable)]
    port_number: i64,
}

impl INetworkSocketAddressNode for NetworkSocketAddressNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}
