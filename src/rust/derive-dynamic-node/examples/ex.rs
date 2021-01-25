use derive_dynamic_node::{NodeDescription,
                          GraplSessionId};
use grapl_graph_descriptions::graph_description::*;

#[derive(NodeDescription, GraplSessionId)]
pub struct SpecialProcess {
    #[grapl(create_time, immutable)]
    pub create_time: u64,
    #[grapl(last_seen_time, immutable)]
    pub seen_at: u64,
    #[grapl(terminate_time, immutable)]
    pub terminate_time: u64,
}

fn main() {}
