use std::time::Duration;

use client_executor::strategy::FibonacciBackoff;
use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::{
        graph_mutation::v1beta1::messages as native,
        client::{
            Connectable,
            Client,
            ClientError,
            Configuration
        },
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
pub struct GraphMutationClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    client: Client<B, GraphMutationServiceClient<tonic::transport::Channel>>,
}

impl <B> GraphMutationClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.graph_mutation.v1beta1.GraphMutationService";

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

    pub async fn create_node(
        &mut self,
        request: native::CreateNodeRequest,
    ) -> Result<native::CreateNodeResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.create_node(request),
        ).await?)
    }

    pub async fn set_node_property(
        &mut self,
        request: native::SetNodePropertyRequest,
    ) -> Result<native::SetNodePropertyResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.set_node_property(request),
        ).await?)
    }

    pub async fn create_edge(
        &mut self,
        request: native::CreateEdgeRequest,
    ) -> Result<native::CreateEdgeResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.create_edge(request),
        ).await?)
    }
}
