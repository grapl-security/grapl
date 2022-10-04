use std::time::Duration;

use client_executor::strategy::FibonacciBackoff;
use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::{
        graph_schema_manager::v1beta1::messages as native,
        client::{Client, Connectable, ClientError, Configuration},
    },
    protobufs::graplinc::grapl::api::graph_schema_manager::v1beta1::graph_schema_manager_service_client::GraphSchemaManagerServiceClient,
};

#[async_trait::async_trait]
impl Connectable
    for GraphSchemaManagerServiceClient<tonic::transport::Channel>
{
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

#[derive(Clone)]
pub struct GraphSchemaManagerClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    client: Client<B, GraphSchemaManagerServiceClient<tonic::transport::Channel>>,
}

impl <B> GraphSchemaManagerClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.graph_schema_manager.v1beta1.GraphSchemaManagerService";

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
        let client = Client::new(configuration)?;

        Ok(Self { client })
    }

    pub async fn deploy_schema(
        &mut self,
        request: native::DeploySchemaRequest,
    ) -> Result<native::DeploySchemaResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.deploy_schema(request),
        ).await?)
    }

    pub async fn get_edge_schema(
        &mut self,
        request: native::GetEdgeSchemaRequest,
    ) -> Result<native::GetEdgeSchemaResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.get_edge_schema(request),
        ).await?)
    }
}
