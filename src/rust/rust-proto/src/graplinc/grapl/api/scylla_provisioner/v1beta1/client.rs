use std::time::Duration;

use client_executor::strategy::FibonacciBackoff;
use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::{
        scylla_provisioner::v1beta1::messages as native,
        client::{
            Client,
            ClientError,
            Configuration,
            Connectable
        },
    },
    protobufs::graplinc::grapl::api::scylla_provisioner::v1beta1::scylla_provisioner_service_client::ScyllaProvisionerServiceClient,
};

#[async_trait::async_trait]
impl Connectable
    for ScyllaProvisionerServiceClient<tonic::transport::Channel>
{
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

pub struct ScyllaProvisionerClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    client: Client<B, ScyllaProvisionerServiceClient<tonic::transport::Channel>>
}


impl <B> ScyllaProvisionerClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.scylla_provisioner.v1beta1.ScyllaProvisionerService";

    pub fn new<A>(
        address: A,
        request_timeout: Duration,
        executor_timeout: Duration,
        concurrency_limit: usize,
        initial_backoff_delay: Duration,
        maximum_backoff_delay: Duration,
    ) -> Result<Self, ClientError>
    where
        A: TryInto<Endpoint>,
    {
        let configuration = Configuration::new(
            Self::SERVICE_NAME,
            address,
            request_timeout,
            executor_timeout,
            concurrency_limit,
            FibonacciBackoff::from_millis(initial_backoff_delay.as_millis())
                .max_delay(maximum_backoff_delay)
                .map(client_executor::strategy::jitter),
        )?;
        let client = Client::new(configuration);

        Ok(Self { client })
    }

    pub async fn provision_graph_for_tenant(
        &mut self,
        request: native::ProvisionGraphForTenantRequest,
    ) -> Result<native::ProvisionGraphForTenantResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.provision_graph_for_tenant(request),
        ).await?)
    }
}
