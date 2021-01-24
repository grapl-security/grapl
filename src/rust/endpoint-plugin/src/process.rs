use derive_dynamic_node::{GraplSessionId,
                          NodeDescription};
use grapl_graph_descriptions::graph_description::*;

#[derive(NodeDescription, GraplSessionId)]
pub struct Process {
    #[allow(dead_code)]
    #[grapl(pseudo_key, immutable)]
    process_id: u64,
    #[allow(dead_code)]
    #[grapl(immutable)]
    process_guid: u64,
    #[allow(dead_code)]
    #[grapl(create_time, immutable)]
    created_timestamp: u64,
    #[allow(dead_code)]
    #[grapl(terminate_time, immutable)]
    terminated_timestamp: u64,
    #[allow(dead_code)]
    #[grapl(last_seen_time, increment)]
    last_seen_timestamp: u64,
    #[allow(dead_code)]
    #[grapl(immutable)]
    process_name: u64,
    #[allow(dead_code)]
    #[grapl(immutable)]
    process_command_line: u64,
    #[allow(dead_code)]
    #[grapl(immutable)]
    operating_system: u64,
    #[allow(dead_code)]
    #[grapl(immutable)]
    asset_id: String,
}

impl IProcessNode for ProcessNode {
    fn get_mut_dynamic_node(&mut self) -> &mut NodeDescription {
        &mut self.dynamic_node
    }

    fn get_dynamic_node(&self) -> &NodeDescription {
        &self.dynamic_node
    }
}
