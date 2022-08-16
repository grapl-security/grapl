use crate::{
    graplinc::grapl::api::graph_query_service::v1beta1::messages::{
        QueryGraphFromUidRequest,
        QueryGraphFromUidResponse,
        QueryGraphWithUidRequest,
        QueryGraphWithUidResponse,
    },
    protobufs::graplinc::grapl::api::graph_query_service::v1beta1::{
        graph_query_service_client::GraphQueryServiceClient,
        QueryGraphFromUidRequest as QueryGraphFromUidRequestProto,
        QueryGraphWithUidRequest as QueryGraphWithUidRequestProto,
    },
    protocol::status::Status,
    SerDeError,
};

#[derive(thiserror::Error, Debug)]
pub enum GraphQueryClientError {
    #[error("Failed to deserialize response {0}")]
    SerDeError(#[from] SerDeError),
    #[error("Status {0}")]
    Status(#[from] Status),
    #[error("ConnectError")]
    ConnectError(tonic::transport::Error),
}

#[derive(Clone)]
pub struct GraphQueryClient {
    inner: GraphQueryServiceClient<tonic::transport::Channel>,
}

impl GraphQueryClient {
    pub async fn connect<T>(endpoint: T) -> Result<Self, GraphQueryClientError>
    where
        T: TryInto<tonic::transport::Endpoint>,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        Ok(GraphQueryClient {
            inner: GraphQueryServiceClient::connect(endpoint)
                .await
                .map_err(GraphQueryClientError::ConnectError)?,
        })
    }

    pub async fn query_graph_with_uid(
        &mut self,
        request: QueryGraphWithUidRequest,
    ) -> Result<QueryGraphWithUidResponse, GraphQueryClientError> {
        let request: QueryGraphWithUidRequestProto = request.into();
        Ok(self
            .inner
            .query_graph_with_uid(request)
            .await
            .map_err(Status::from)?
            .into_inner()
            .try_into()?)
    }
    pub async fn query_graph_from_uid(
        &mut self,
        request: QueryGraphFromUidRequest,
    ) -> Result<QueryGraphFromUidResponse, GraphQueryClientError> {
        let request: QueryGraphFromUidRequestProto = request.into();
        Ok(self
            .inner
            .query_graph_from_uid(request)
            .await
            .map_err(Status::from)?
            .into_inner()
            .try_into()?)
    }
}
