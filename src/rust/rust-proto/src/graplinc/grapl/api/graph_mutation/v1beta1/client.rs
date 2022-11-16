use std::fmt::Debug;

use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::{
        client::{
            Client,
            ClientError,
            Connectable,
            WithClient,
        },
        graph_mutation::v1beta1::messages as native,
    },
    protobufs::graplinc::grapl::api::graph_mutation::v1beta1::graph_mutation_service_client::GraphMutationServiceClient,
};

#[async_trait::async_trait]
impl Connectable for GraphMutationServiceClient<tonic::transport::Channel> {
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

#[derive(Clone)]
pub struct GraphMutationClient {
    client: Client<GraphMutationServiceClient<tonic::transport::Channel>>,
}

impl Debug for GraphMutationClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphMutationClient")
            .field("client", &self.client)
            .finish()
    }
}

impl WithClient<GraphMutationServiceClient<tonic::transport::Channel>> for GraphMutationClient {
    fn with_client(client: Client<GraphMutationServiceClient<tonic::transport::Channel>>) -> Self {
        Self { client }
    }
}

impl GraphMutationClient {
    pub async fn create_node(
        &mut self,
        request: native::CreateNodeRequest,
    ) -> Result<native::CreateNodeResponse, ClientError> {
        self.client
            .execute(
                request,
                None,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.create_node(request).await },
            )
            .await
    }

    pub async fn set_node_property(
        &mut self,
        request: native::SetNodePropertyRequest,
    ) -> Result<native::SetNodePropertyResponse, ClientError> {
        self.client
            .execute(
                request,
                None,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.set_node_property(request).await },
            )
            .await
    }

    pub async fn create_edge(
        &mut self,
        request: native::CreateEdgeRequest,
    ) -> Result<native::CreateEdgeResponse, ClientError> {
        self.client
            .execute(
                request,
                None,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.create_edge(request).await },
            )
            .await
    }
}
