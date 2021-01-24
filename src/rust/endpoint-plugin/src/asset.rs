use derive_dynamic_node::{GraplStaticId,
                          NodeDescription};
use grapl_graph_descriptions::graph_description::*;

#[derive(NodeDescription, GraplStaticId)]
pub struct Asset {
    #[grapl(static_id, immutable)]
    #[allow(dead_code)]
    hostname: String,
    #[grapl(static_id, immutable)]
    #[allow(dead_code)]
    launch_time: u64,
    #[grapl(static_id, increment)]
    #[allow(dead_code)]
    last_seen_time: u64,
}

impl IAssetNode for AssetNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}
