extern crate derive_dynamic_node;
extern crate graph_descriptions;

use derive_dynamic_node::DynamicNode as DN;

#[derive(DN)]
pub struct Ec2Instance2 {
    arn: String,
    launch_time: u64,
}

fn main() {}
