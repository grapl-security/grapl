use derive_dynamic_node::{
    GraplStaticId,
    NodeDescription,
};
use rust_proto_new::graplinc::grapl::api::graph::v1beta1::{
    IdStrategy,
    NodeDescription,
    NodeProperty,
    Static,
};

#[derive(NodeDescription, GraplStaticId)]
pub struct IpAddress {
    #[grapl(static_id, immutable)]
    ip_address: String,
    #[grapl(decrement)]
    first_seen_timestamp: u64,
    #[grapl(increment)]
    last_seen_timestamp: u64,
}

impl IIpAddressNode for IpAddressNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}
