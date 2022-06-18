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
use scylla::{
    IntoTypedRows,
    Session,
};

use crate::{
    graph_query::{
        EdgeRow,
        GraphQuery,
        StringCmp,
        StringField,
    },
    graph_view::Graph,
    node_view::Node,
    visited::Visited,
};

type RcCell<T> = Rc<RefCell<T>>;

#[derive(Clone)]
pub struct InnerNodeQuery {
    pub query_id: u64,
    pub(crate) node_type: NodeType,
    pub string_filters: HashMap<PropertyName, Vec<Vec<StringCmp>>>,
}

impl InnerNodeQuery {
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


    fn select_node_properties(&self, uid: Uid, tenant_id: uuid::Uuid) -> Vec<String> {
        let uid = uid.as_i64();
        let mut selects = Vec::new();
        let node_type = &self.node_type;
        // todo: tenant_id?
        for prop_name in self.string_filters.keys() {
            // todo: use prepared statements!
            let select = format!(
                r#"
                SELECT populated_field, value FROM immutable_string_index
                WHERE uid = {uid} AND node_type = '{node_type}' AND populated_field = '{prop_name}'
                LIMIT 1
                ALLOW FILTERING;
            "#
            );
            selects.push(select);
        }
        selects
    }

    #[tracing::instrument(skip(self, session))]
    pub async fn fetch_node_properties(
        &self,
        uid: Uid,
        tenant_id: uuid::Uuid,
        session: Arc<Session>,
    ) -> Result<Option<Vec<StringField>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let selects = self.select_node_properties(uid, tenant_id);

        let mut rows: Vec<StringField> = vec![];

        for select in selects {
            println!("selecting: {select}");
            if let Some(next_rows) = session.query(select, &[]).await?.rows {
                if next_rows.is_empty() {
                    return Ok(None);
                }
                debug_assert_eq!(next_rows.len(), 1);

                for row in next_rows.into_typed::<(String, String)>() {
                    let row = row?;
                    let row = StringField {
                        uid,
                        populated_field: PropertyName::try_from(row.0).expect("todo"),
                        value: row.1,
                    };
                    rows.push(row);
                }
            } else {
                // todo: this is actually an error for SELECT
                return Ok(None);
            }
        }

        let mut filter_names: HashSet<_> = self.string_filters.keys().collect();

        for row in rows.iter() {
            filter_names.remove(&row.populated_field);
        }

        if !filter_names.is_empty() {
            // some values didn't exist, not a match
            return Ok(None);
        }

        Ok(Some(rows))
    }

    #[tracing::instrument(skip(self, graph_query, session))]
    pub async fn fetch_edges(
        &self,
        uid: Uid,
        graph_query: &GraphQuery,
        tenant_id: uuid::Uuid,
        session: Arc<Session>,
    ) -> Result<
        Option<HashMap<EdgeName, Vec<EdgeRow>>>,
        Box<dyn std::error::Error + Send + Sync + 'static>,
    > {
        let mut edge_rows = HashMap::new();
        for ((src_id, edge_name), edges) in graph_query.edges.iter() {
            if *src_id != self.query_id {
                continue;
            }
            let edge_query = format!(
                r#"
                SELECT f_edge_name, r_edge_name, destination_uid
                FROM edge
                WHERE source_uid = {uid} AND f_edge_name = '{edge_name}'
                ALLOW FILTERING;
            "#,
                uid = uid.as_i64()
            );
            let mut rows = vec![];
            if let Some(next_rows) = session.query(edge_query, &[]).await?.rows {
                // property is not indexed - no match
                if next_rows.is_empty() {
                    return Ok(None);
                }
                for row in next_rows.into_typed::<(String, String, i64)>() {
                    let row = row?;
                    let row = EdgeRow {
                        source_uid: uid,
                        f_edge_name: EdgeName::try_from(row.0).expect("todo"),
                        r_edge_name: EdgeName::try_from(row.1).expect("todo"),
                        destination_uid: Uid::from_i64(row.2).expect("todo"),
                        tenant_id: tenant_id.clone(),
                    };
                    rows.push(row);
                }
            }
            if rows.is_empty() {
                return Ok(None);
            }
            println!("edge name {}, rows {:?}", edge_name, rows);
            edge_rows.insert(edge_name.to_owned(), rows);
        }

        Ok(Some(edge_rows))
    }

    // todo: Handle cycles
    #[async_recursion]
    pub async fn fetch_node_with_edges(
        &self,
        graph_query: &GraphQuery,
        uid: Uid,
        tenant_id: uuid::Uuid,
        session: Arc<Session>,
        visited: Visited,
    ) -> Result<Option<Graph>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        if visited.get_short_circuit() {
            return Ok(None);
        }

        let mut node = Node::new(uid, self.query_id);

        let node_indices = self
            .fetch_node_properties(uid, tenant_id, session.clone())
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
        assert!(
            !node_indices.is_empty(),
            "node_indices should never be empty - this is a bug"
        );

        // fetch the edges for the uid
        let edges = match self
            .fetch_edges(uid, graph_query, tenant_id, session.clone())
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

                // When we support multiple edge filters we'll add that logic here

                let mut any = false;
                for edge_row in edge_rows {
                    // we can do this in parallel
                    let neighbors = match edge_query
                        .fetch_node_with_edges(
                            graph_query,
                            edge_row.destination_uid,
                            tenant_id,
                            session.clone(),
                            visited.clone(),
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

pub struct NodeQuery {
    pub query_id: u64,
    graph: Option<RcCell<GraphQuery>>,
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
        let mut graph = self.graph.take().unwrap();

        let graph = Rc::get_mut(&mut graph).unwrap();
        graph.take()
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
