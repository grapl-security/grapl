use std::time::Duration;

use client_executor::strategy::FibonacciBackoff;
use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::{
        plugin_work_queue::v1beta1 as native,
        client::{
            Connectable,
            ClientError,
            Client,
            Configuration,
        },
    },
    protobufs::graplinc::grapl::api::plugin_work_queue::v1beta1::plugin_work_queue_service_client::PluginWorkQueueServiceClient,
};

#[async_trait::async_trait]
impl Connectable for PluginWorkQueueServiceClient<tonic::transport::Channel> {
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

#[derive(Clone)]
pub struct PluginWorkQueueClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    client: Client<B, PluginWorkQueueServiceClient<tonic::transport::Channel>>,
}

impl <B> PluginWorkQueueClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.plugin_work_queue.v1beta1.PluginWorkQueueService";

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

    /// Adds a new execution job for a generator
    #[tracing::instrument(skip(self, request), err)]
    pub async fn push_execute_generator(
        &mut self,
        request: native::PushExecuteGeneratorRequest,
    ) -> Result<native::PushExecuteGeneratorResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.push_execute_generator(request),
        ).await?)
    }

    /// Adds a new execution job for an analyzer
    #[tracing::instrument(skip(self, request), err)]
    pub async fn push_execute_analyzer(
        &mut self,
        request: native::PushExecuteAnalyzerRequest,
    ) -> Result<native::PushExecuteAnalyzerResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.push_execute_analyzer(request),
        ).await?)
    }

    /// Retrieves a new execution job for a generator
    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_execute_generator(
        &mut self,
        request: native::GetExecuteGeneratorRequest,
    ) -> Result<native::GetExecuteGeneratorResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.get_execute_generator(request),
        ).await?)
    }

    /// Retrieves a new execution job for an analyzer
    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_execute_analyzer(
        &mut self,
        request: native::GetExecuteAnalyzerRequest,
    ) -> Result<native::GetExecuteAnalyzerResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.get_execute_analyzer(request),
        ).await?)
    }

    /// Acknowledges the completion of a generator job
    #[tracing::instrument(skip(self, request), err)]
    pub async fn acknowledge_generator(
        &mut self,
        request: native::AcknowledgeGeneratorRequest,
    ) -> Result<native::AcknowledgeGeneratorResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.acknowledge_generator(request),
        ).await?)
    }

    /// Acknowledges the completion of an analyzer job
    #[tracing::instrument(skip(self, request), err)]
    pub async fn acknowledge_analyzer(
        &mut self,
        request: native::AcknowledgeAnalyzerRequest,
    ) -> Result<native::AcknowledgeAnalyzerResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.acknowledge_analyzer(request),
        ).await?)
    }
}
