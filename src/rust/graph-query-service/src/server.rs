use std::{
    collections::HashMap,
    sync::Arc,
};

use futures::future::try_join_all;
use rust_proto_new::{
    graplinc::grapl::{
        api::graph_query::v1beta1::{
            messages::{
                NodeQuery as NodeQueryProto,
                OrStringFilters,
                QueryGraphFromNodeRequest,
                QueryGraphFromNodeResponse,
                QueryGraphWithNodeRequest,
                QueryGraphWithNodeResponse,
                StringFilter,
                StringOperation,
            },
            server::GraphQueryApi,
        },
        common::v1beta1::types::EdgeName,
    },
    protocol::status::Status,
};
use scylla::Session;
use tonic::{
    Request,
    Response,
};

use crate::{
    graph_query::{
        GraphQuery,
        StringCmp,
    },
    node_query::NodeQuery,
    visited::Visited,
};

#[derive(thiserror::Error, Debug)]
pub enum GraphQueryError {
    #[error("todo")]
    Todo(&'static str),
}

impl From<GraphQueryError> for Status {
    fn from(e: GraphQueryError) -> Self {
        Status::unimplemented("foo")
    }
}

#[derive(Clone)]
pub struct GraphQueryApiImpl {
    scylla_client: Arc<Session>,
}

#[async_trait::async_trait]
impl GraphQueryApi for GraphQueryApiImpl {
    type Error = GraphQueryError;

    async fn query_graph_with_uid(
        &self,
        request: QueryGraphWithNodeRequest,
    ) -> Result<QueryGraphWithNodeResponse, GraphQueryError> {
        let node_uid = request.node_uid;

        let node_query = request.node_query;
        let edge_mapping = request.edge_mapping;
        let graph_query = convert_to_root_query(node_query, &edge_mapping);
        let f = graph_query
            .query_graph(node_uid, request.tenant_id, self.scylla_client.clone())
            .await;

        Err(GraphQueryError::Todo(""))
    }

    async fn query_graph_from_uid(
        &self,
        request: QueryGraphFromNodeRequest,
    ) -> Result<QueryGraphFromNodeResponse, GraphQueryError> {
        let node_uid = request.node_uid;

        let node_query = request.node_query;
        let edge_mapping = request.edge_mapping;
        let graph_query = convert_to_root_query(node_query, &edge_mapping);
        let node_query = &graph_query.nodes[&graph_query.root_query_id];
        let visited = Visited::new();
        node_query
            .fetch_node_with_edges(
                &graph_query,
                node_uid,
                request.tenant_id,
                self.scylla_client.clone(),
                visited,
            )
            .await
            .expect("error: todo")
            .expect("no match");

        Err(GraphQueryError::Todo(""))
    }
}

fn convert_to_root_query(
    node_query_proto: NodeQueryProto,
    edge_mapping: &HashMap<EdgeName, EdgeName>,
) -> GraphQuery {
    let mut root_node = NodeQuery::root(node_query_proto.node_type.clone());
    convert_node_query(&mut root_node, node_query_proto, &edge_mapping);
    let graph_query = root_node.build();
    drop(root_node);
    graph_query
}

fn convert_node_query(
    node_query: &mut NodeQuery,
    node_query_proto: NodeQueryProto,
    edge_mapping: &HashMap<EdgeName, EdgeName>,
) {
    for (prop_name, prop_filters) in node_query_proto.string_filters {
        let prop_filters = prop_filters.into();
        node_query.overwrite_string_comparisons(prop_name, prop_filters);
    }

    for (edge_name, edge_filters) in node_query_proto.edge_filters {
        let reverse_edge_name = edge_mapping[&edge_name].clone();

        for edge_filter in edge_filters.node_queries {
            node_query.with_edge_to(
                edge_name.clone(),
                reverse_edge_name.clone(),
                edge_filter.node_type.clone(),
                move |neighbor| {
                    convert_node_query(neighbor, edge_filter, edge_mapping);
                },
            );
        }

        // convert_edge_filters(node_query, edge_filters, edge_mapping);
        // node_query.overwrite_edges(edge_name, reverse_edge_name, edge_filters);
    }
}

// fn convert_edge_filters(node_query: &mut NodeQuery, edge_filters: EdgeFiltersProto, edge_mapping: &HashMap<String, String>) {
//     let mut edge_filters = edge_filters.node_queries;
//
//     for edge_filter in edge_filters {
//         node_query.add_
//         convert_node_query(node_query, edge_filter, edge_mapping);
//
//     }
// }
