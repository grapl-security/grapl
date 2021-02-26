use derive_dynamic_node::{GraplSessionId,
                          NodeDescription};
use grapl_graph_descriptions::graph_description::*;

#[derive(NodeDescription, GraplSessionId)]
pub struct SpecialProcess {
    #[grapl(pseudo_key, immutable)]
    pub process_id: u64,
    #[grapl(immutable)]
    pub process_name: String,
    #[grapl(create_time, immutable)]
    pub create_time: u64,
    #[grapl(last_seen_time, immutable)]
    pub last_seen_at: u64,
    #[grapl(terminate_time, immutable)]
    pub terminate_time: u64,
}

fn main() {}
