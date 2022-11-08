use rust_proto::graplinc::grapl::api::graph::v1beta1::{
    IdentifiedGraph,
    ImmutableUintProp,
};

use crate::find_node::FindNode;

/// Look for some nodes we'd expect to see from 36_eventlog.xml being node-identified
pub fn events_36lines_node_identity_predicate(identified_graph: IdentifiedGraph) -> bool {
    let parent_process =
        identified_graph.find_node("process_id", ImmutableUintProp { prop: 6132 }.into());

    let child_process =
        identified_graph.find_node("process_id", ImmutableUintProp { prop: 5752 }.into());

    match (parent_process, child_process) {
        (Some(parent_process), Some(child_process)) => {
            let parent_to_child_edge = identified_graph
                .edges
                .get(&parent_process.uid)
                .iter()
                .flat_map(|edge_list| edge_list.edges.iter())
                .find(|edge| edge.to_uid == child_process.uid)
                .expect("missing edge from parent to child");

            parent_to_child_edge.edge_name == "children"
        }
        _ => false,
    }
}
