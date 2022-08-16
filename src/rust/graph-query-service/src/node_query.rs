use std::{
    cell::RefCell,
    rc::Rc,
};

use async_recursion::async_recursion;
use rust_proto::graplinc::grapl::{
    api::graph_query_service::v1beta1::messages::{
        AndStringFilters,
        GraphQuery,
        GraphView,
        NodePropertiesView,
        NodePropertyQuery,
        OrStringFilters,
        QueryId,
        StrCmp,
        StringProperties,
    },
    common::v1beta1::types::{
        EdgeName,
        NodeType,
        PropertyName,
        Uid,
    },
};
use rustc_hash::{
    FxHashMap,
    FxHashSet,
};

use crate::{
    property_query::{
        EdgeRow,
        PropertyQueryError,
        PropertyQueryExecutor,
        StringField,
    },
    short_circuit::ShortCircuit,
    visited::Visited,
};

#[derive(thiserror::Error, Debug)]
pub enum NodeQueryError {
    #[error("PropertyQueryError {0}")]
    PropertyQueryError(#[from] PropertyQueryError),
}

pub(crate) fn match_property(
    node_properties_query: &NodePropertyQuery,
    property_name: &PropertyName,
    property_value: &str,
) -> bool {
    'outer: for or_filters in
        &node_properties_query.string_filters[property_name].and_string_filters
    {
        for and_filter in &or_filters.string_filters {
            match StrCmp::from(and_filter) {
                StrCmp::Eq(to, negated) => match (negated, property_value == to) {
                    (false, false) => continue 'outer,
                    (true, true) => continue 'outer,
                    (_, _) => (),
                },
                StrCmp::Contains(to, negated) => match (negated, property_value.contains(&to)) {
                    (false, false) => continue 'outer,
                    (true, true) => continue 'outer,
                    (_, _) => (),
                },
                StrCmp::Has => (),
            };
        }
        return true;
    }

    false
}

#[tracing::instrument(skip(node_properties_query, property_query_executor))]
pub async fn fetch_node_properties(
    node_properties_query: &NodePropertyQuery,
    uid: Uid,
    tenant_id: uuid::Uuid,
    property_query_executor: PropertyQueryExecutor,
) -> Result<Option<Vec<StringField>>, NodeQueryError> {
    let mut fields = vec![];
    if !node_properties_query.string_filters.is_empty() {
        let mut filter_names: FxHashSet<_> = node_properties_query.string_filters.keys().collect();

        for prop_name in node_properties_query.string_filters.keys() {
            let property = property_query_executor
                .get_immutable_string(tenant_id, uid, prop_name)
                .await?;
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

#[tracing::instrument(skip(node_properties_query, graph_query, property_query_executor))]
pub async fn fetch_edges(
    node_properties_query: &NodePropertyQuery,
    uid: Uid,
    graph_query: &GraphQuery,
    tenant_id: uuid::Uuid,
    property_query_executor: PropertyQueryExecutor,
) -> Result<Option<FxHashMap<EdgeName, Vec<EdgeRow>>>, NodeQueryError> {
    let mut edge_rows = FxHashMap::default();
    for (src_id, edge_name) in graph_query.edge_filters.keys() {
        if *src_id != node_properties_query.query_id {
            continue;
        }

        let rows = property_query_executor
            .get_edges(tenant_id, uid, edge_name)
            .await?;

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
    node_properties_query: &NodePropertyQuery,
    graph_query: &GraphQuery,
    uid: Uid,
    tenant_id: uuid::Uuid,
    property_query_executor: PropertyQueryExecutor,
    visited: Visited,
    x_short_circuit: ShortCircuit,
    root_node_uid: &mut Option<Uid>,
) -> Result<Option<GraphView>, NodeQueryError> {
    if visited.get_short_circuit() || x_short_circuit.get_short_circuit() {
        return Ok(None);
    }

    let mut node = NodePropertiesView::new(
        uid,
        node_properties_query.node_type.clone(),
        StringProperties::default(),
    );

    let node_properties = fetch_node_properties(
        node_properties_query,
        uid,
        tenant_id,
        property_query_executor.clone(),
    )
    .await?;

    let node_properties = match node_properties {
        None => {
            visited.set_short_circuit();
            return Ok(None);
        }
        Some(node_properties) => node_properties,
    };

    for node_property in node_properties.iter() {
        if match_property(
            node_properties_query,
            &node_property.populated_field,
            &node_property.value,
        ) {
            node.add_string_property(
                node_property.populated_field.clone(),
                node_property.value.clone(),
            );
        } else {
            visited.set_short_circuit();
            return Ok(None);
        }
    }

    if node_properties_query.query_id == graph_query.root_query_id {
        *root_node_uid = Some(uid);
    }

    let mut graph = GraphView::default();
    graph.add_node(node);

    tracing::debug!(
        message = "Retrieved node indices",
        count = node_properties.len(),
    );

    if x_short_circuit.get_short_circuit() {
        return Ok(None);
    }

    // fetch the edges for the uid
    let edges = match fetch_edges(
        node_properties_query,
        uid,
        graph_query,
        tenant_id,
        property_query_executor.clone(),
    )
    .await?
    {
        Some(edges) => edges,
        None => {
            visited.set_short_circuit();
            return Ok(None);
        }
    };

    for ((src_id, edge_name), edge_queries) in graph_query.edge_filters.iter() {
        if *src_id != node_properties_query.query_id {
            continue;
        }
        let edge_rows = &edges[edge_name];

        for edge_query_id in edge_queries {
            let edge_query = &graph_query.node_property_queries[edge_query_id];
            // we have to check the reverse edge as well
            if visited.check_and_add(
                node_properties_query.query_id,
                edge_name.clone(),
                edge_query.query_id,
            ) {
                continue;
            }

            if visited.check_and_add(
                edge_query.query_id,
                graph_query.edge_map[edge_name].to_owned(),
                node_properties_query.query_id,
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
                let neighbors = match fetch_node_with_edges(
                    edge_query,
                    graph_query,
                    edge_row.destination_uid,
                    tenant_id,
                    property_query_executor.clone(),
                    visited.clone(),
                    x_short_circuit.clone(),
                    root_node_uid,
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

// Note: Different from the rust_proto NodeQuery.
pub struct NodeQuery {
    pub query_id: QueryId,
    graph: Option<Rc<RefCell<GraphQuery>>>,
}

impl NodeQuery {
    pub fn root(node_type: NodeType) -> Self {
        let query_id = QueryId::default();
        let inner_query = NodePropertyQuery {
            query_id,
            node_type,
            int_filters: Default::default(),
            string_filters: Default::default(),
            uid_filters: Default::default(),
        };
        let mut node_property_queries = FxHashMap::default();
        node_property_queries.insert(query_id, inner_query);

        let graph = GraphQuery {
            root_query_id: query_id,
            node_property_queries,
            edge_filters: Default::default(),
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
        comparisons: impl Into<AndStringFilters>,
    ) -> &mut Self {
        let mut inner = self.graph.as_mut().unwrap().borrow_mut();
        inner
            .node_property_queries
            .get_mut(&self.query_id)
            .unwrap()
            .string_filters
            .entry(property_name)
            .or_insert_with(OrStringFilters::new)
            .push(comparisons.into());
        drop(inner);
        self
    }

    pub fn overwrite_string_comparisons(
        &mut self,
        property_name: PropertyName,
        comparisons: OrStringFilters,
    ) {
        let mut inner = self.graph.as_mut().unwrap().borrow_mut();
        inner
            .node_property_queries
            .get_mut(&self.query_id)
            .unwrap()
            .string_filters
            .insert(property_name, comparisons);
    }

    pub fn with_shared_edge(
        &mut self,
        edge_name: EdgeName,
        reverse_edge_name: EdgeName,
        node: NodePropertyQuery,
        init_edge: impl FnOnce(&mut Self),
    ) -> &mut Self {
        let neighbor_query_id = node.query_id;
        {
            let graph = self.graph.as_mut().unwrap();
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
            let graph = self.graph.as_mut().unwrap();
            let mut graph = graph.borrow_mut();
            graph
                .edge_filters
                .entry((self.query_id, edge_name.clone()))
                .or_insert(FxHashSet::default())
                .insert(neighbor_query_id);
            graph
                .edge_filters
                .entry((neighbor_query_id, reverse_edge_name.clone()))
                .or_insert(FxHashSet::default())
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
        init_edge: impl FnOnce(&mut Self),
    ) -> &mut Self {
        let new_neighbor_id = QueryId::default();

        {
            let graph = self.graph.as_mut().unwrap();
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
            let graph = self.graph.as_mut().unwrap();
            let mut graph = graph.borrow_mut();
            graph
                .edge_filters
                .entry((self.query_id, edge_name.clone()))
                .or_insert(FxHashSet::default())
                .insert(new_neighbor_id);
            graph
                .edge_filters
                .entry((new_neighbor_id, reverse_edge_name.clone()))
                .or_insert(FxHashSet::default())
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
            root_query_id: Default::default(),
            node_property_queries: Default::default(),
            edge_filters: Default::default(),
            edge_map: Default::default(),
        })
    }
}
