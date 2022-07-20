use std::sync::Arc;
use scylla::CachingSession;
use rust_proto::{
    graplinc::grapl::api::graph_query_service::v1beta1::{
        messages::{
            GraphQuery,
            GraphView,
            QueryGraphFromNodeRequest,
            QueryGraphFromNodeResponse,
            QueryGraphWithNodeRequest,
            QueryGraphWithNodeResponse,
        },
        server::GraphQueryApi,
    },
    protocol::status::Status,
};

use crate::{
    graph_query::query_graph,
    node_query::fetch_node_with_edges,
    property_query::PropertyQueryExecutor,
    short_circuit::ShortCircuit,
    visited::Visited,
};

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
pub struct GraphQueryService {
    property_query_executor: PropertyQueryExecutor,
}

impl GraphQueryService {
    pub fn new(scylla_client: Arc<CachingSession>) -> Self {
        Self {
            property_query_executor: PropertyQueryExecutor::new(scylla_client),
        }
    }
}

#[async_trait::async_trait]
impl GraphQueryApi for GraphQueryService {
    type Error = GraphQueryError;

    async fn query_graph_with_uid(
        &self,
        request: QueryGraphWithNodeRequest,
    ) -> Result<QueryGraphWithNodeResponse, GraphQueryError> {
        let node_uid = request.node_uid;

        let graph_query: GraphQuery = request.graph_query;
        let graph = query_graph(
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
        let node_query = &graph_query
            .node_property_queries
            .get(&graph_query.root_query_id)
            .unwrap();

        let visited = Visited::new();
        let x_short_circuit = ShortCircuit::new();
        let graph = fetch_node_with_edges(
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
