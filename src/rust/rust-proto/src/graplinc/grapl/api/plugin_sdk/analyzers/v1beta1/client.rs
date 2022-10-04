use std::time::Duration;

use client_executor::strategy::FibonacciBackoff;
use tonic::transport::Endpoint;
use tracing::instrument;

use crate::{
    graplinc::grapl::api::{
        plugin_sdk::analyzers::v1beta1::messages as native,
        client::{
            Connectable,
            Client,
            ClientError,
            Configuration,
        },
    },
    protobufs::graplinc::grapl::api::plugin_sdk::analyzers::v1beta1::analyzer_service_client::AnalyzerServiceClient,
};

#[async_trait::async_trait]
impl Connectable for AnalyzerServiceClient<tonic::transport::Channel> {
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

#[derive(Clone)]
pub struct AnalyzerClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    client: Client<B, AnalyzerServiceClient<tonic::transport::Channel>>,
}

impl <B> AnalyzerClient<B>
where
    B: IntoIterator<Item = Duration> + Clone,
{
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.plugin_registry.v1beta1.AnalyzerService";

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

    /// retrieve the plugin corresponding to the given plugin_id
    #[instrument(skip(self, request), err)]
    pub async fn run_analyzer(
        &mut self,
        request: native::RunAnalyzerRequest,
    ) -> Result<native::RunAnalyzerResponse, ClientError> {
        Ok(self.client.execute(
            request,
            |status, request| status.code() == tonic::Code::Unavailable,
            10,
            |client, request| client.run_analyzer(request),
        ).await?)
    }
}
