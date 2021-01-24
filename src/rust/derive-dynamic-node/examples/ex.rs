use derive_dynamic_node::{DynamicNode,
                          GraplSessionId};
use grapl_graph_descriptions::graph_description::*;

// #[derive(DynamicNode)]
// pub struct Ec2Instance2 {
//     #[allow(dead_code)] // DynamicNode requires fields
//     arn: String,
//     #[allow(dead_code)] // DynamicNode requires fields
//     launch_time: u64,
// }

#[derive(DynamicNode, GraplSessionId)]
pub struct SpecialProcess {
    #[grapl(creation_time)]
    pub create_time: u64,
    #[grapl(last_seen_time)]
    pub seen_at: u64,
    #[grapl(terminate_time)]
    pub terminate_time: u64,
}

fn main() {}
