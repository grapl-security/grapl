use rust_proto::graplinc::grapl::api::graph::v1beta1::{
    IdentifiedGraph,
    IdentifiedNode,
    MergedGraph,
    MergedNode,
    Property,
};

pub trait FindNode<T> {
    fn find_node<'a>(&'a self, o_p_name: &str, o_p_value: Property) -> Option<&'a T>;
}

impl FindNode<IdentifiedNode> for IdentifiedGraph {
    fn find_node<'a>(&'a self, o_p_name: &str, o_p_value: Property) -> Option<&'a IdentifiedNode> {
        self.nodes.values().find(|n| {
            n.properties.iter().any(|(p_name, p_value)| {
                p_name.as_str() == o_p_name && p_value.property.clone() == o_p_value
            })
        })
    }
}

impl FindNode<MergedNode> for MergedGraph {
    fn find_node<'a>(&'a self, o_p_name: &str, o_p_value: Property) -> Option<&'a MergedNode> {
        self.nodes.values().find(|n| {
            n.properties.iter().any(|(p_name, p_value)| {
                p_name.as_str() == o_p_name && p_value.property.clone() == o_p_value
            })
        })
    }
}
