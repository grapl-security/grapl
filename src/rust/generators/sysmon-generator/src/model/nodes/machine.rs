use derive_dynamic_node::{
    GraplStaticId,
    NodeDescription,
};
use rust_proto::graph_descriptions::*;

#[derive(Debug, Clone, PartialEq, Hash, NodeDescription, GraplStaticId)]
struct Machine {
    #[grapl(static_id, immutable)]
    machine_id: String,
    #[grapl(immutable)]
    hostname: String,
    #[grapl(immutable)]
    os: String,
}

impl IMachineNode for MachineNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}
