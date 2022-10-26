use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::{
        client::{
            client_impl,
            Client,
            ClientError,
            Connectable,
        },
        graph_query::v1beta1::messages as native,
    },
    protobufs::graplinc::grapl::api::graph_query::v1beta1::graph_query_service_client::GraphQueryServiceClient,
};

#[async_trait::async_trait]
impl Connectable for GraphQueryServiceClient<tonic::transport::Channel> {
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

#[derive(Clone)]
pub struct GraphQueryClient {
    client: Client<GraphQueryServiceClient<tonic::transport::Channel>>,
}

impl client_impl::WithClient<GraphQueryServiceClient<tonic::transport::Channel>>
    for GraphQueryClient
{
    const SERVICE_NAME: &'static str = "graplinc.grapl.api.graph_query.v1beta1.GraphQueryService";

    fn with_client(client: Client<GraphQueryServiceClient<tonic::transport::Channel>>) -> Self {
        Self { client }
    }
}

impl GraphQueryClient {
    pub async fn query_graph_with_uid(
        &mut self,
        request: native::QueryGraphWithUidRequest,
    ) -> Result<native::QueryGraphWithUidResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.query_graph_with_uid(request).await },
            )
            .await
    }

    pub async fn query_graph_from_uid(
        &mut self,
        request: native::QueryGraphFromUidRequest,
    ) -> Result<native::QueryGraphFromUidResponse, ClientError> {
        self.client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.query_graph_from_uid(request).await },
            )
            .await
    }
}
