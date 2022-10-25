use std::sync::Arc;

use rust_proto::{
    graplinc::grapl::api::graph_query::v1beta1::{
        messages::{
            GraphQuery,
            MatchedGraphWithUid,
            MaybeMatchWithUid,
            NoMatchWithUid,
            QueryGraphFromUidRequest,
            QueryGraphFromUidResponse,
            QueryGraphWithUidRequest,
            QueryGraphWithUidResponse,
        },
        server::GraphQueryApi,
    },
    protocol::status::Status,
};
use scylla::CachingSession;

use crate::{
    graph_query::{
        query_graph,
        GraphQueryError,
    },
    node_query::{
        fetch_node_with_edges,
        NodeQueryError,
    },
    property_query::PropertyQueryExecutor,
    short_circuit::ShortCircuit,
    visited::Visited,
};

#[derive(thiserror::Error, Debug)]
pub enum GraphQueryServiceError {
    #[error("GraphQueryError {0}")]
    GraphQueryError(#[from] GraphQueryError),
    #[error("NodeQueryError {0}")]
    NodeQueryError(#[from] NodeQueryError),
}

impl From<GraphQueryServiceError> for Status {
    fn from(gqs_err: GraphQueryServiceError) -> Self {
        type GQSErr = GraphQueryServiceError;
        match gqs_err {
            GQSErr::GraphQueryError(e) => Status::unknown(e.to_string()),
            GQSErr::NodeQueryError(e) => Status::unknown(e.to_string()),
        }
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
    type Error = GraphQueryServiceError;

    async fn query_graph_with_uid(
        &self,
        request: QueryGraphWithUidRequest,
    ) -> Result<QueryGraphWithUidResponse, GraphQueryServiceError> {
        let node_uid = request.node_uid;

        let graph_query: GraphQuery = request.graph_query;
        let graph = query_graph(
            &graph_query,
            node_uid,
            request.tenant_id,
            self.property_query_executor.clone(),
        )
        .await?;

        let (matched_graph, root_uid) = match graph {
            Some((graph, root_uid)) => (graph, root_uid),
            None => {
                return Ok(QueryGraphWithUidResponse {
                    maybe_match: MaybeMatchWithUid::Missed(NoMatchWithUid {}),
                })
            }
        };

        Ok(QueryGraphWithUidResponse {
            maybe_match: MaybeMatchWithUid::Matched(MatchedGraphWithUid {
                matched_graph,
                root_uid,
            }),
        })
    }

    async fn query_graph_from_uid(
        &self,
        request: QueryGraphFromUidRequest,
    ) -> Result<QueryGraphFromUidResponse, GraphQueryServiceError> {
        let node_uid = request.node_uid;

        let graph_query: GraphQuery = request.graph_query;
        let node_query = &graph_query
            .node_property_queries
            .get(&graph_query.root_query_id)
            .unwrap();

        let visited = Visited::new();
        let x_short_circuit = ShortCircuit::new();
        let graph = fetch_node_with_edges(
            node_query,
            &graph_query,
            node_uid,
            request.tenant_id,
            self.property_query_executor.clone(),
            visited,
            x_short_circuit,
            &mut None,
        )
        .await?;

        Ok(QueryGraphFromUidResponse {
            matched_graph: graph,
        })
    }
}
