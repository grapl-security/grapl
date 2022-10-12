use rust_proto::{
    graplinc::grapl::api::{
        graph_query::v1beta1::{
            client::{
                GraphQueryClient,
                GraphQueryClientError,
            },
            messages as non_proxy_messages,
        },
        graph_query_proxy::v1beta1::{
            messages::{
                QueryGraphFromUidRequest,
                QueryGraphFromUidResponse,
                QueryGraphWithUidRequest,
                QueryGraphWithUidResponse,
            },
            server::GraphQueryProxyApi,
        },
    },
    protocol::status::Status,
};

#[derive(thiserror::Error, Debug)]
pub enum GraphQueryProxyError {
    #[error("GraphQueryClientError {0}")]
    GraphQueryClientError(#[from] GraphQueryClientError),
}

impl From<GraphQueryProxyError> for Status {
    fn from(gqp_err: GraphQueryProxyError) -> Self {
        match gqp_err {
            GraphQueryProxyError::GraphQueryClientError(e) => Status::unknown(e.to_string()),
        }
    }
}

#[derive(Clone)]
pub struct GraphQueryProxy {
    tenant_id: uuid::Uuid,
    graph_query_client: GraphQueryClient,
}

impl GraphQueryProxy {
    pub fn new(tenant_id: uuid::Uuid, graph_query_client: GraphQueryClient) -> Self {
        Self {
            tenant_id,
            graph_query_client,
        }
    }
}

#[async_trait::async_trait]
impl GraphQueryProxyApi for GraphQueryProxy {
    type Error = GraphQueryProxyError;

    #[tracing::instrument(skip(self), err)]
    async fn query_graph_with_uid(
        &self,
        request: QueryGraphWithUidRequest,
    ) -> Result<QueryGraphWithUidResponse, GraphQueryProxyError> {
        let request = non_proxy_messages::QueryGraphWithUidRequest {
            tenant_id: self.tenant_id,
            graph_query: request.graph_query,
            node_uid: request.node_uid,
        };
        let mut graph_query_client = self.graph_query_client.clone();
        Ok(graph_query_client
            .query_graph_with_uid(request)
            .await?
            .into())
    }

    #[tracing::instrument(skip(self), err)]
    async fn query_graph_from_uid(
        &self,
        request: QueryGraphFromUidRequest,
    ) -> Result<QueryGraphFromUidResponse, GraphQueryProxyError> {
        let request = non_proxy_messages::QueryGraphFromUidRequest {
            tenant_id: self.tenant_id,
            graph_query: request.graph_query,
            node_uid: request.node_uid,
        };
        let mut graph_query_client = self.graph_query_client.clone();
        Ok(graph_query_client
            .query_graph_from_uid(request)
            .await?
            .into())
    }
}
