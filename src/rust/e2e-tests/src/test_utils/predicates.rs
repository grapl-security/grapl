use rust_proto::graplinc::grapl::api::graph::v1beta1::{
    IdentifiedGraph,
    ImmutableUintProp,
};

// use rust_proto::graplinc::grapl::api::plugin_sdk::analyzers::v1beta1::messages::Updates;
use crate::test_utils::find_node::FindNode;

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

// pub fn events_36lines_merged_graph_predicate(updates: Updates) -> bool {
//     let parent_process =
//         merged_graph.find_node("process_id", ImmutableUintProp { prop: 6132 }.into());
//
//     let child_process =
//         merged_graph.find_node("process_id", ImmutableUintProp { prop: 5752 }.into());
//
//     // NOTE: here, unlike node-identifier, we expect the edge
//     // connecting the parent and child proceses to be *absent*
//     // in the message emitted to the merged-graphs topic. The
//     // reason for this is that downstream services (analyzers)
//     // don't operate on edges, just nodes. So the view of the
//     // graph diverges at the graph-merger--we now tell one story
//     // in our Kafka messages and a totally different story in
//     // Dgraph. This is confusing and we should fix it:
//     //
//     // https://app.zenhub.com/workspaces/grapl-6036cbd36bacff000ef314f2/issues/grapl-security/issue-tracker/950
//     match (parent_process, child_process) {
//         (Some(parent_process), Some(child_process)) => !merged_graph
//             .edges
//             .get(parent_process.get_node_key())
//             .iter()
//             .flat_map(|edge_list| edge_list.edges.iter())
//             .any(|edge| edge.to_node_key == child_process.get_node_key()),
//         _ => false,
//     }
// }
