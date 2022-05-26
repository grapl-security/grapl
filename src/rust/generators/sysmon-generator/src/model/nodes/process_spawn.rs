use derive_dynamic_node::{
    GraplStaticId,
    NodeDescription,
};
use rust_proto::graph_descriptions::*;

#[derive(Debug, Clone, PartialEq, Hash, NodeDescription, GraplStaticId)]
struct ProcessSpawn {
    #[grapl(immutable)]
    timestamp: i64,
    #[grapl(immutable)]
    cmdline: String,
    #[grapl(immutable)]
    current_directory: String,
    #[grapl(immutable)]
    uid: i64,
    #[grapl(immutable)]
    user: String,
    #[grapl(immutable)]
    parent_user: String,


    // identity-only fields
    #[grapl(static_id, immutable)]
    parent_guid: String,
    #[grapl(static_id, immutable)]
    child_guid: String,
}

impl IProcessSpawnNode for ProcessSpawnNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}
