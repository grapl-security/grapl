use rust_proto::graplinc::grapl::api::graph::v1beta1::{
    IdentifiedGraph,
    IdentifiedNode,
    ImmutableUintProp,
    Property,
};

fn find_node<'a>(
    graph: &'a IdentifiedGraph,
    o_p_name: &str,
    o_p_value: Property,
) -> Option<&'a IdentifiedNode> {
    graph.nodes.values().find(|n| {
        n.properties.iter().any(|(p_name, p_value)| {
            p_name.as_str() == o_p_name && p_value.property.clone() == o_p_value
        })
    })
}

/// Look for some nodes we'd expect to see from events6 being node-identified
pub fn events6_node_identity_predicate(identified_graph: &IdentifiedGraph) -> bool {
    let parent_process = find_node(
        identified_graph,
        "process_id",
        ImmutableUintProp { prop: 6132 }.into(),
    )
    .expect("parent process missing");

    let child_process = find_node(
        identified_graph,
        "process_id",
        ImmutableUintProp { prop: 5752 }.into(),
    )
    .expect("child process missing");

    let parent_to_child_edge = identified_graph
        .edges
        .get(parent_process.get_node_key())
        .iter()
        .flat_map(|edge_list| edge_list.edges.iter())
        .find(|edge| edge.to_node_key == child_process.get_node_key())
        .expect("missing edge from parent to child");

    parent_to_child_edge.edge_name == "children"
}
