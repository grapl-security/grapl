extern crate derive_dynamic_node;
extern crate graph_descriptions;

use graph_descriptions::graph_description::*;
use derive_dynamic_node::DynamicNode;

#[derive(DynamicNode)]
pub struct Ec2Instance2 {
    arn: String,
    launch_time: u64,
}

fn main() {}
