use std::{
    cell::{
        Ref,
        RefCell,
    },
    collections::{
        HashMap,
        HashSet,
    },
    ops::Deref,
    rc::{
        Rc,
        Weak,
    },
    sync::Arc,
};

use async_recursion::async_recursion;
use futures::future::try_join_all;
use rust_proto_new::graplinc::grapl::common::v1beta1::types::{
    EdgeName,
    NodeType,
    PropertyName,
    Uid,
};
use scylla::{CachingSession, IntoTypedRows, Session};

use crate::{graph_query::{
    GraphQuery,
    StringCmp,
}, graph_view::Graph, node_view::Node, visited::Visited};
use crate::property_query::{PropertyQueryExecutor, StringField, EdgeRow};
use crate::short_circuit::ShortCircuit;


#[derive(Clone)]
pub struct InnerNodeQuery {
    pub query_id: u64,
    pub(crate) node_type: NodeType,
    pub string_filters: HashMap<PropertyName, Vec<Vec<StringCmp>>>,
}

impl InnerNodeQuery {
    pub fn new(node_type: NodeType) -> Self {
        let query_id = std::cmp::max(rand::random(), 1);

        Self {
            query_id,
            node_type,
            string_filters: HashMap::new(),
        }
    }

    pub(crate) fn match_property(
        &self,
        property_name: &PropertyName,
        property_value: &str,
    ) -> bool {
        'outer: for or_filters in &self.string_filters[property_name] {
            for and_filter in or_filters {
                match and_filter {
                    StringCmp::Eq(to, negated) => match (negated, property_value == to) {
                        (false, false) => continue 'outer,
                        (true, true) => continue 'outer,
                        (_, _) => (),
                    },
                    StringCmp::Contains(to, negated) => {
                        match (negated, property_value.contains(to)) {
                            (false, false) => continue 'outer,
                            (true, true) => continue 'outer,
                            (_, _) => (),
                        }
                    }
                    StringCmp::Has => (),
                };
            }
            return true;
        }

        false
    }

    #[tracing::instrument(skip(self, property_query_executor))]
    pub async fn fetch_node_properties(
        &self,
        uid: Uid,
        tenant_id: uuid::Uuid,
        property_query_executor: PropertyQueryExecutor,
    ) -> Result<Option<Vec<StringField>>, Box<dyn std::error::Error + Send + Sync + 'static>> {

        let mut fields = vec![];
        if !self.string_filters.is_empty() {
            let mut filter_names: HashSet<_> = self.string_filters.keys().collect();

            for prop_name in self.string_filters.keys() {
                let property = property_query_executor.get_immutable_string(
                    tenant_id,
                    uid,
                    &self.node_type,
                    prop_name,
                ).await?;
                match property {
                    Some(p) => fields.push(p),
                    None => return Ok(None),
                }
                filter_names.remove(prop_name);
            }


            if !filter_names.is_empty() {
                // some values didn't exist, not a match
                return Ok(None);
            }
        }

        Ok(Some(fields))
    }

    #[tracing::instrument(skip(self, graph_query, property_query_executor))]
    pub async fn fetch_edges(
        &self,
        uid: Uid,
        graph_query: &GraphQuery,
        tenant_id: uuid::Uuid,
        property_query_executor: PropertyQueryExecutor,
    ) -> Result<
        Option<HashMap<EdgeName, Vec<EdgeRow>>>,
        Box<dyn std::error::Error + Send + Sync + 'static>,
    > {

        let mut edge_rows = HashMap::new();
        for (src_id, edge_name) in graph_query.edges.keys() {
            if *src_id != self.query_id {
                continue;
            }

            let rows = property_query_executor.get_edges(
                tenant_id,
                uid,
                edge_name,
            ).await?;

            let rows = match rows {
                None => return Ok(None),
                Some(rows) => rows,
            };
            debug_assert!(!rows.is_empty());

            println!("edge name {}, rows {:?}", edge_name, rows);
            edge_rows.insert(edge_name.to_owned(), rows);
        }

        Ok(Some(edge_rows))
    }

    #[async_recursion]
    pub async fn fetch_node_with_edges(
        &self,
        graph_query: &GraphQuery,
        uid: Uid,
        tenant_id: uuid::Uuid,
        property_query_executor: PropertyQueryExecutor,
        visited: Visited,
        x_short_circuit: ShortCircuit,
    ) -> Result<Option<Graph>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        if visited.get_short_circuit() || x_short_circuit.get_short_circuit() {
            return Ok(None);
        }

        let mut node = Node::new(uid, self.node_type.clone(), self.query_id);

        let node_indices = self
            .fetch_node_properties(uid, tenant_id, property_query_executor.clone())
            .await?;

        let node_indices = match node_indices {
            None => {
                visited.set_short_circuit();
                return Ok(None);
            }
            Some(node_indices) => node_indices,
        };

        for node_index in node_indices.iter() {
            if self.match_property(&node_index.populated_field, &node_index.value) {
                node.add_string_property(
                    node_index.populated_field.clone(),
                    node_index.value.clone(),
                );
            } else {
                visited.set_short_circuit();
                return Ok(None);
            }
        }

        let mut graph = Graph::default();
        graph.add_node(node);

        tracing::debug!(
            message = "Retrieved node indices",
            count = node_indices.len(),
        );

        if x_short_circuit.get_short_circuit() {
            return Ok(None);
        }

        // fetch the edges for the uid
        let edges = match self
            .fetch_edges(uid, graph_query, tenant_id, property_query_executor.clone())
            .await?
        {
            Some(edges) => edges,
            None => {
                visited.set_short_circuit();
                return Ok(None);
            }
        };

        for ((src_id, edge_name), edge_queries) in graph_query.edges.iter() {
            if *src_id != self.query_id {
                continue;
            }
            let edge_rows = &edges[edge_name];

            for edge_query_id in edge_queries {
                let edge_query = &graph_query.nodes[&edge_query_id];
                // we have to check the reverse edge as well
                if visited.check_and_add(self.query_id, edge_name.clone(), edge_query.query_id) {
                    continue;
                }

                if visited.check_and_add(
                    edge_query.query_id,
                    graph_query.edge_map[edge_name].to_owned(),
                    self.query_id,
                ) {
                    continue;
                }

                // When we support 'OR' logic on edges we'll add that logic here

                let mut any = false;
                for edge_row in edge_rows {
                    // we can do this in parallel
                    if x_short_circuit.get_short_circuit() {
                        return Ok(None);
                    }
                    let neighbors = match edge_query
                        .fetch_node_with_edges(
                            graph_query,
                            edge_row.destination_uid,
                            tenant_id,
                            property_query_executor.clone(),
                            visited.clone(),
                            x_short_circuit.clone(),
                        )
                        .await?
                    {
                        Some(neighbors) => neighbors,
                        None => continue,
                    };
                    any = true;
                    for neighbor in neighbors.nodes.keys() {
                        graph.add_edge(uid, edge_name.to_owned(), *neighbor);
                        graph.add_edge(*neighbor, graph_query.edge_map[edge_name].to_owned(), uid);
                    }
                    graph.merge(neighbors);
                }
                if !any {
                    // if a given query has no matches, return
                    visited.set_short_circuit();
                    return Ok(None);
                }
            }
        }

        Ok(Some(graph))
    }
}

// Note: Different from the rust_proto_new NodeQuery.
pub struct NodeQuery {
    pub query_id: u64,
    graph: Option<Rc<RefCell<GraphQuery>>>,
}

impl NodeQuery {
    pub fn root(node_type: NodeType) -> Self {
        let query_id = std::cmp::max(rand::random(), 1);
        let inner_query = InnerNodeQuery {
            query_id,
            node_type,
            string_filters: HashMap::new(),
        };

        let graph = GraphQuery {
            root_query_id: query_id,
            nodes: HashMap::from([(query_id, inner_query)]),
            edges: Default::default(),
            edge_map: Default::default(),
        };

        Self {
            query_id,
            graph: Some(Rc::new(RefCell::new(graph))),
        }
    }

    pub fn with_string_comparisons(
        &mut self,
        property_name: PropertyName,
        comparisons: impl Into<Vec<StringCmp>>,
    ) -> &mut Self {
        let mut inner = self.graph.as_mut().unwrap().borrow_mut();
        inner
            .nodes
            .get_mut(&self.query_id)
            .unwrap()
            .string_filters
            .entry(property_name)
            .or_insert(Vec::new())
            .push(comparisons.into());
        drop(inner);
        self
    }

    pub fn overwrite_string_comparisons(
        &mut self,
        property_name: PropertyName,
        comparisons: Vec<Vec<StringCmp>>,
    ) {
        let mut inner = self.graph.as_mut().unwrap().borrow_mut();
        inner
            .nodes
            .get_mut(&self.query_id)
            .unwrap()
            .string_filters
            .insert(property_name, comparisons);
    }

    pub fn with_shared_edge(
        &mut self,
        edge_name: EdgeName,
        reverse_edge_name: EdgeName,
        node: InnerNodeQuery,
        init_edge: impl FnOnce(&mut Self) -> (),
    ) -> &mut Self {
        let neighbor_query_id = node.query_id;
        {
            let mut graph = self.graph.as_mut().unwrap();
            let mut graph = graph.borrow_mut();
            graph.merge_node(node);
        }
        let mut neighbor = Self {
            query_id: neighbor_query_id,
            graph: self.graph.clone(),
        };

        init_edge(&mut neighbor);
        neighbor.graph = None;
        {
            let mut graph = self.graph.as_mut().unwrap();
            let mut graph = graph.borrow_mut();
            graph
                .edges
                .entry((self.query_id, edge_name.clone()))
                .or_insert(HashSet::new())
                .insert(neighbor_query_id);
            graph
                .edges
                .entry((neighbor_query_id, reverse_edge_name.clone()))
                .or_insert(HashSet::new())
                .insert(self.query_id);
            graph
                .edge_map
                .insert(edge_name.clone(), reverse_edge_name.clone());
            graph.edge_map.insert(reverse_edge_name, edge_name);
        }
        self
    }


    pub fn with_edge_to(
        &mut self,
        edge_name: EdgeName,
        reverse_edge_name: EdgeName,
        node_type: NodeType,
        init_edge: impl FnOnce(&mut Self) -> (),
    ) -> &mut Self {
        let new_neighbor_id = std::cmp::max(rand::random(), 1);

        {
            let mut graph = self.graph.as_mut().unwrap();
            let mut graph = graph.borrow_mut();
            graph.add_node(new_neighbor_id, node_type);
        }

        let mut neighbor = Self {
            query_id: new_neighbor_id,
            graph: self.graph.clone(),
        };

        init_edge(&mut neighbor);
        neighbor.graph = None;
        {
            let mut graph = self.graph.as_mut().unwrap();
            let mut graph = graph.borrow_mut();
            graph
                .edges
                .entry((self.query_id, edge_name.clone()))
                .or_insert(HashSet::new())
                .insert(new_neighbor_id);
            graph
                .edges
                .entry((new_neighbor_id, reverse_edge_name.clone()))
                .or_insert(HashSet::new())
                .insert(self.query_id);
            graph
                .edge_map
                .insert(edge_name.clone(), reverse_edge_name.clone());
            graph.edge_map.insert(reverse_edge_name, edge_name);
        }
        self
    }

    pub fn build(&mut self) -> GraphQuery {
        // This will panic if you have not attached this node to a graph ie: it must be attached
        // to a root node somewhere
        let mut graph = self.graph.take().unwrap();

        let graph = Rc::get_mut(&mut graph).unwrap();
        graph.replace(GraphQuery {
            root_query_id: 0,
            nodes: Default::default(),
            edges: Default::default(),
            edge_map: Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_graph_query() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let query = NodeQuery::root("foo".try_into()?)
            .with_edge_to(
                "forward".try_into()?,
                "reverse".try_into()?,
                "Process".try_into()?,
                |neighbor| {
                    neighbor.with_string_comparisons(
                        "process_name".try_into().unwrap(),
                        [StringCmp::eq("chrome.exe", false)],
                    );
                },
            )
            .build();

        // query.query_graph(1, uuid::Uuid::new_v4(), "session_id", vec![], "graph_id").await?;

        Ok(())
    }
}
