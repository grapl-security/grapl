use std::time::Duration;

use client_executor::{
    Executor,
    ExecutorConfig,
};

use crate::{
    create_proto_client,
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
    protocol::{
        service_client::ConnectError,
        status::Status,
    },
    SerDeError,
};

#[derive(thiserror::Error, Debug)]
pub enum GraphQueryClientError {
    #[error("Failed to deserialize response {0}")]
    SerDeError(#[from] SerDeError),
    #[error("Status {0}")]
    Status(#[from] Status),
    #[error("Rejected")]
    Rejected,
    #[error("Elapsed")]
    Elapsed,
}

impl From<client_executor::Error<GraphQueryClientError>> for GraphQueryClientError {
    fn from(e: client_executor::Error<GraphQueryClientError>) -> Self {
        match e {
            client_executor::Error::Rejected => GraphQueryClientError::Rejected,
            client_executor::Error::Elapsed => GraphQueryClientError::Elapsed,
            client_executor::Error::Inner(e) => e,
        }
    }
}

#[derive(Clone)]
pub struct GraphQueryClient {
    inner: GraphQueryServiceClient<tonic::transport::Channel>,
    executor: Executor,
}

impl GraphQueryClient {
    pub async fn connect<T>(endpoint: T) -> Result<Self, ConnectError>
    where
        T: Clone,
        tonic::transport::Endpoint: TryFrom<T> + Clone,
        <T as TryInto<tonic::transport::Endpoint>>::Error:
            std::error::Error + Send + Sync + 'static,
    {
        let executor = Executor::new(ExecutorConfig::new(Duration::from_secs(30)));
        let proto_client = create_proto_client!(
            executor,
            GraphQueryServiceClient<tonic::transport::Channel>,
            endpoint,
        );

        Ok(GraphQueryClient {
            inner: proto_client,
            executor,
        })
    }

    pub async fn query_graph_with_uid(
        &mut self,
        request: QueryGraphWithUidRequest,
    ) -> Result<QueryGraphWithUidResponse, GraphQueryClientError> {
        let backoff = client_executor::strategy::FibonacciBackoff::from_millis(100)
            .max_delay(Duration::from_millis(5000))
            .map(client_executor::strategy::jitter);
        let num_retries = 10;
        let request: QueryGraphWithUidRequestProto = request.into();

        let proto_client = self.inner.clone();
        let r = self
            .executor
            .spawn(backoff.take(num_retries), || {
                let request = request.clone();
                let mut proto_client = proto_client.clone();
                async move {
                    Ok(proto_client
                        .query_graph_with_uid(request)
                        .await
                        .map_err(Status::from)?
                        .into_inner()
                        .try_into()?)
                }
            })
            .await?;

        Ok(r)
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
