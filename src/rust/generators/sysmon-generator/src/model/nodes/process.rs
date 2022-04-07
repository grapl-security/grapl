use derive_dynamic_node::{
    GraplStaticId,
    NodeDescription,
};
use rust_proto::graph_descriptions::*;

#[derive(Debug, Clone, PartialEq, Hash, NodeDescription, GraplStaticId)]
struct Process {
    #[grapl(immutable)]
    pid: i64,
    #[grapl(static_id, immutable)]
    guid: String,
    #[grapl(immutable)]
    created_timestamp: i64,
    #[grapl(immutable)]
    cmdline: String,
    #[grapl(immutable)]
    image: String,
    #[grapl(immutable)]
    current_directory: String,
    #[grapl(immutable)]
    user: String,
}

impl IProcessNode for ProcessNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}
