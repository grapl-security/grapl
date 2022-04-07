use derive_dynamic_node::{
    GraplStaticId,
    NodeDescription,
};
use rust_proto::graph_descriptions::*;

#[derive(Debug, Clone, PartialEq, Hash, NodeDescription, GraplStaticId)]
struct File {
    #[grapl(static_id, immutable)]
    machine_id: String,
    #[grapl(static_id, immutable)]
    path: String,
    // #[grapl(last_seen_time, increment)]
    // last_seen_timestamp: i64,
}

impl IFileNode for FileNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}
