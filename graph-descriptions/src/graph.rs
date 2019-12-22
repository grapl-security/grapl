use std::collections::HashMap;

use graph_description::{Edge, EdgeList, GeneratedSubgraphs, Node};
use graph_description::Graph;
use node::NodeT;

impl Graph {
    pub fn new(timestamp: u64) -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            timestamp
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    pub fn merge(&mut self, other: &Graph) {
        self.edges.extend(other.edges.clone());

        for (node_key, other_node) in other.nodes.iter() {
            self.nodes
                .entry(node_key.clone())
                .and_modify(|node| {
                    node.merge(other_node);
                })
                .or_insert_with(|| other_node.clone());
        }
    }

    pub fn add_node<N>(&mut self, node: N)
        where N: Into<Node>
    {
        let node = node.into();
        let key = node.clone_node_key();

        self.nodes.insert(key.to_string(), node);
        self.edges
            .entry(key)
            .or_insert_with(|| {
                EdgeList { edges: vec![] }
            });
    }


    pub fn with_node<N>(mut self, node: N) -> Graph
        where N: Into<Node>
    {
        self.add_node(node);
        self
    }

    pub fn add_edge(&mut self,
                    edge_name: impl Into<String>,
                    from: impl Into<String>,
                    to: impl Into<String>)
    {
        let from = from.into();
        let to = to.into();
        let edge_name = edge_name.into();
        let edge = Edge {
            from: from.clone(),
            to,
            edge_name
        };

        self.edges
            .entry(from)
            .or_insert_with(|| {
                EdgeList { edges: Vec::with_capacity(1) }
            })
            .edges.push(edge);
    }
}


impl GeneratedSubgraphs {
    pub fn new(subgraphs: Vec<Graph>) -> GeneratedSubgraphs {
        GeneratedSubgraphs {
            subgraphs
        }
    }
}
