use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::{
        scylla_provisioner::v1beta1::messages as native,
        client::{
            Client,
            ClientError,
            Connectable,
            WithClient,
        },
    },
    protobufs::graplinc::grapl::api::scylla_provisioner::v1beta1::scylla_provisioner_service_client::ScyllaProvisionerServiceClient,
};

#[async_trait::async_trait]
impl Connectable for ScyllaProvisionerServiceClient<tonic::transport::Channel> {
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

pub struct ScyllaProvisionerClient {
    client: Client<ScyllaProvisionerServiceClient<tonic::transport::Channel>>,
}

impl WithClient<ScyllaProvisionerServiceClient<tonic::transport::Channel>>
    for ScyllaProvisionerClient
{
    fn with_client(
        client: Client<ScyllaProvisionerServiceClient<tonic::transport::Channel>>,
    ) -> Self {
        Self { client }
    }
}

impl ScyllaProvisionerClient {
    pub async fn provision_graph_for_tenant(
        &mut self,
        request: native::ProvisionGraphForTenantRequest,
    ) -> Result<native::ProvisionGraphForTenantResponse, ClientError> {
        self
            .client
            .execute(
                request,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.provision_graph_for_tenant(request).await },
            )
            .await
    }
}
