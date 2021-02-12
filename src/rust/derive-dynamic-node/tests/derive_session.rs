use derive_dynamic_node::{DynamicNode,
                          GraplSessionId,};
use grapl_graph_descriptions::graph_description::*;

#[derive(DynamicNode, GraplSessionId)]
pub struct SpecialProcess {
    #[grapl(created_time)]
    pub create_time: u64,
    #[grapl(last_seen_time)]
    pub seen_at: u64,
    #[grapl(terminated_time)]
    pub terminate_time: u64,
    #[grapl(pseudo_key)]
    pub process_id: u64,
}

impl ISpecialProcessNode for SpecialProcessNode {
        fn get_mut_dynamic_node(&mut self) -> &mut DynamicNode {
            &mut self.dynamic_node
        }

        fn get_dynamic_node(&self) -> &DynamicNode {
            &self.dynamic_node
        }
}

#[test]
fn test_session() {
    let last_seen_timestamp = 1234;
    let mut special_proc = SpecialProcessNode::new(
        SpecialProcessNode::session_strategy(), last_seen_timestamp
    );

    special_proc.with_create_time(1234u64);
}
