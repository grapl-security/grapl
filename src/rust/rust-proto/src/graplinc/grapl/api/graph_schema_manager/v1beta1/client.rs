use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::{
        graph_schema_manager::v1beta1::messages as native,
        client::{
            Client,
            Connectable,
            ClientError,
            WithClient,
        },
    },
    protobufs::graplinc::grapl::api::graph_schema_manager::v1beta1::graph_schema_manager_service_client::GraphSchemaManagerServiceClient,
};

#[async_trait::async_trait]
impl Connectable for GraphSchemaManagerServiceClient<tonic::transport::Channel> {
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

#[derive(Clone)]
pub struct GraphSchemaManagerClient {
    client: Client<GraphSchemaManagerServiceClient<tonic::transport::Channel>>,
}

impl WithClient<GraphSchemaManagerServiceClient<tonic::transport::Channel>>
    for GraphSchemaManagerClient
{
    fn with_client(
        client: Client<GraphSchemaManagerServiceClient<tonic::transport::Channel>>,
    ) -> Self {
        Self { client }
    }
}

impl GraphSchemaManagerClient {
    pub async fn deploy_schema(
        &mut self,
        request: native::DeploySchemaRequest,
    ) -> Result<native::DeploySchemaResponse, ClientError> {
        self.client
            .execute(
                request,
                None,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.deploy_schema(request).await },
            )
            .await
    }

    pub async fn get_edge_schema(
        &mut self,
        request: native::GetEdgeSchemaRequest,
    ) -> Result<native::GetEdgeSchemaResponse, ClientError> {
        self.client
            .execute(
                request,
                None,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.get_edge_schema(request).await },
            )
            .await
    }
}
