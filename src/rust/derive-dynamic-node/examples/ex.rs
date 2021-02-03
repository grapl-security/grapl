use derive_dynamic_node::DynamicNode;
use grapl_graph_descriptions::graph_description::*;

#[derive(DynamicNode)]
pub struct Ec2Instance2 {
    #[allow(dead_code)] // DynamicNode requires fields
    arn: String,
    #[allow(dead_code)] // DynamicNode requires fields
    launch_time: u64,
}

fn main() {}
