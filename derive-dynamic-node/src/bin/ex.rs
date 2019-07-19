#![allow(warnings)]
extern crate derive_dynamic_node;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

use grapl_graph_descriptions::graph_description::DynamicNode;

use serde_derive::Deserialize;
use serde_derive::Serialize;

use grapl_graph_descriptions::graph_description::*;

use derive_dynamic_node::{DynamicNode, GraplStaticId};

fn read_log() -> &'static [u8] { unimplemented!() }

#[derive(Clone, Debug, Deserialize)]
struct InstanceDetails {
    #[serde(rename = "arn")]
    arn: String,
    #[serde(rename = "imageId")]
    image_id: String,
    #[serde(rename = "instanceId")]
    instance_id: String,
    #[serde(rename = "instanceState")]
    instance_state: String,
    #[serde(rename = "instanceType")]
    instance_type: String,
    #[serde(rename = "launchTime")]
    launch_time: u64,
}

#[derive(DynamicNode, GraplStaticId)]
pub struct AwsEc2Instance {
    #[grapl(static_id)]
    arn: String,
    launch_time: u64,
}

impl IAwsEc2InstanceNode for AwsEc2InstanceNode {
    fn get_mut_dynamic_node(&mut self) -> &mut DynamicNode {
        &mut self.dynamic_node
    }

    fn with_arn(&mut self, arn: impl Into<String>) -> &mut Self {
        info!("custom arn handler");
        self.get_mut_dynamic_node().set_property("arn", arn.into());
        self
    }
}

fn main() {
    let raw_guard_duty_alert = read_log();

    let log: InstanceDetails = serde_json::from_slice(raw_guard_duty_alert).unwrap();

    let mut ec2 = AwsEc2InstanceNode::new(AwsEc2InstanceNode::static_strategy(), log.launch_time);
    ec2.with_arn(log.arn).with_launch_time(log.launch_time);

    let mut graph = GraphDescription::new(log.launch_time);
    graph.add_node(ec2);
}
