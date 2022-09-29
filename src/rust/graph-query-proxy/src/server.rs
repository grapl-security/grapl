use rust_proto::{
    graplinc::grapl::api::graph_query_service::v1beta1::{
        client::{
            GraphQueryClient,
            GraphQueryClientError,
        },
        messages::{
            QueryGraphFromUidRequest,
            QueryGraphFromUidResponse,
            QueryGraphWithUidRequest,
            QueryGraphWithUidResponse,
        },
        server::GraphQueryApi,
    },
    protocol::status::Status,
};

#[derive(thiserror::Error, Debug)]
pub enum GraphQueryProxyError {
    #[error("GraphQueryClientError {0}")]
    GraphQueryClientError(#[from] GraphQueryClientError),
}

impl From<GraphQueryProxyError> for Status {
    fn from(gqs_err: GraphQueryProxyError) -> Self {
        match gqs_err {
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
impl GraphQueryApi for GraphQueryProxy {
    type Error = GraphQueryProxyError;

    #[tracing::instrument(skip(self), err)]
    async fn query_graph_with_uid(
        &self,
        mut request: QueryGraphWithUidRequest,
    ) -> Result<QueryGraphWithUidResponse, GraphQueryProxyError> {
        request.tenant_id = self.tenant_id;
        let mut graph_query_client = self.graph_query_client.clone();
        Ok(graph_query_client.query_graph_with_uid(request).await?)
    }

    #[tracing::instrument(skip(self), err)]
    async fn query_graph_from_uid(
        &self,
        mut request: QueryGraphFromUidRequest,
    ) -> Result<QueryGraphFromUidResponse, GraphQueryProxyError> {
        request.tenant_id = self.tenant_id;
        let mut graph_query_client = self.graph_query_client.clone();
        Ok(graph_query_client.query_graph_from_uid(request).await?)
    }
}
