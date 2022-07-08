use rust_proto::{
    graplinc::grapl::{
        api::graph_query_service::v1beta1::{
            messages::{
                GraphView,
            },
            server::GraphQueryApi,
        },
    },
    protocol::status::Status,
};
use rust_proto::graplinc::grapl::api::graph_query_service::v1beta1::messages::{GraphQuery, QueryGraphFromNodeRequest, QueryGraphFromNodeResponse, QueryGraphWithNodeRequest, QueryGraphWithNodeResponse};

use crate::{
    property_query::PropertyQueryExecutor,
    short_circuit::ShortCircuit,
    visited::Visited,
};
use crate::graph_query::query_graph;
use crate::node_query::fetch_node_with_edges;

#[derive(thiserror::Error, Debug)]
pub enum GraphQueryError {
    #[error("todo")]
    Todo(&'static str),
}

impl From<GraphQueryError> for Status {
    fn from(_e: GraphQueryError) -> Self {
        Status::unimplemented("foo")
    }
}

#[derive(Clone)]
pub struct GraphQueryApiImpl {
    property_query_executor: PropertyQueryExecutor,
}

#[async_trait::async_trait]
impl GraphQueryApi for GraphQueryApiImpl {
    type Error = GraphQueryError;

    async fn query_graph_with_uid(
        &self,
        request: QueryGraphWithNodeRequest,
    ) -> Result<QueryGraphWithNodeResponse, GraphQueryError> {
        let node_uid = request.node_uid;

        let graph_query: GraphQuery = request.graph_query;
        let graph =
            query_graph(
                &graph_query,
                node_uid,
                request.tenant_id,
                self.property_query_executor.clone(),
            )
            .await;

        let (graph, root_uid) = graph.unwrap().unwrap();

        let graph_view = GraphView::from(graph);

        Ok(QueryGraphWithNodeResponse {
            matched_graph: graph_view,
            root_uid,
        })
    }

    async fn query_graph_from_uid(
        &self,
        request: QueryGraphFromNodeRequest,
    ) -> Result<QueryGraphFromNodeResponse, GraphQueryError> {
        let node_uid = request.node_uid;

        let graph_query: GraphQuery = request.graph_query;
        let node_query = &graph_query.node_property_queries.get(&graph_query.root_query_id).unwrap();

        let visited = Visited::new();
        let x_short_circuit = ShortCircuit::new();
        let graph =
            fetch_node_with_edges(
                &node_query,
                &graph_query,
                node_uid,
                request.tenant_id,
                self.property_query_executor.clone(),
                visited,
                x_short_circuit,
                &mut None,
            )
            .await
            .expect("error: todo")
            .expect("no match");

        let graph_view = GraphView::from(graph);

        Ok(QueryGraphFromNodeResponse {
            matched_graph: graph_view,
        })
    }
}

// fn convert_to_root_query(
//     node_query_proto: NodeQueryProto,
//     edge_mapping: &HashMap<EdgeName, EdgeName>,
// ) -> GraphQuery {
//     let mut root_node = NodeQuery::root(node_query_proto.node_type.clone());
//     convert_node_query(&mut root_node, node_query_proto, &edge_mapping);
//     let graph_query = root_node.build();
//     drop(root_node);
//     graph_query
// }
//
// fn convert_node_query(
//     node_query: &mut NodeQuery,
//     node_query_proto: NodeQueryProto,
//     edge_mapping: &HashMap<EdgeName, EdgeName>,
// ) {
//     for (prop_name, prop_filters) in node_query_proto.string_filters {
//         let prop_filters = prop_filters.into();
//         node_query.overwrite_string_comparisons(prop_name, prop_filters);
//     }
//
//     for (edge_name, edge_filters) in node_query_proto.edge_filters {
//         let reverse_edge_name = edge_mapping[&edge_name].clone();
//
//         for edge_filter in edge_filters.node_queries {
//             node_query.with_edge_to(
//                 edge_name.clone(),
//                 reverse_edge_name.clone(),
//                 edge_filter.node_type.clone(),
//                 move |neighbor| {
//                     convert_node_query(neighbor, edge_filter, edge_mapping);
//                 },
//             );
//         }
//     }
// }
