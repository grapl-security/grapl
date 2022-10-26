use std::{
    fmt::Debug,
    time::Duration,
};

use client_executor::{
    strategy::FibonacciBackoff,
    Executor,
    ExecutorConfig,
};
use figment::{
    Figment,
    Provider,
};
use futures::{
    Future,
    Stream,
};
use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;
use tonic::transport::Endpoint;
use tracing::Instrument;

use super::protocol::{
    healthcheck::{
        client::HealthcheckClient,
        HealthcheckError,
    },
    status::Status,
};
use crate::{
    serde_impl::ProtobufSerializable,
    SerDeError,
};

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("endpoint is not properly formatted {0}")]
    BadEndpoint(#[from] tonic::transport::Error),

    #[error("failed to extract configuration from provider {0}")]
    ConfigurationFailed(#[from] figment::Error),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientConfiguration {
    address: String,
    request_timeout: Duration,
    executor_timeout: Duration,
    concurrency_limit: usize,
    initial_backoff_delay: Duration,
    maximum_backoff_delay: Duration,
    connect_timeout: Duration,
    connect_retries: usize,
    connect_initial_backoff_delay: Duration,
    connect_maximum_backoff_delay: Duration,
}

impl ClientConfiguration {
    pub fn new(
        address: String,
        request_timeout: Duration,
        executor_timeout: Duration,
        concurrency_limit: usize,
        initial_backoff_delay: Duration,
        maximum_backoff_delay: Duration,
        connect_timeout: Duration,
        connect_retries: usize,
        connect_initial_backoff_delay: Duration,
        connect_maximum_backoff_delay: Duration,
    ) -> Self {
        Self {
            address,
            request_timeout,
            executor_timeout,
            concurrency_limit,
            initial_backoff_delay,
            maximum_backoff_delay,
            connect_timeout,
            connect_retries,
            connect_initial_backoff_delay,
            connect_maximum_backoff_delay,
        }
    }

    pub fn from<T>(provider: T) -> Result<Self, ConfigurationError>
    where
        T: Provider,
    {
        Ok(Figment::from(provider).extract()?)
    }

    pub(crate) fn endpoint(&self) -> Result<Endpoint, ConfigurationError> {
        Ok(Endpoint::try_from(self.address.clone())?
            .concurrency_limit(self.concurrency_limit)
            .timeout(self.request_timeout))
    }

    pub(crate) fn backoff_strategy(&self) -> impl Iterator<Item = Duration> + Clone {
        FibonacciBackoff::from_millis(
            1000 * self.initial_backoff_delay.as_secs()
                + self.initial_backoff_delay.subsec_millis() as u64,
        )
        .max_delay(self.maximum_backoff_delay)
        .map(client_executor::strategy::jitter)
    }

    pub(crate) fn connect_backoff_strategy(&self) -> impl Iterator<Item = Duration> + Clone {
        FibonacciBackoff::from_millis(
            1000 * self.connect_initial_backoff_delay.as_secs()
                + self.connect_initial_backoff_delay.subsec_millis() as u64,
        )
        .max_delay(self.connect_maximum_backoff_delay)
        .map(client_executor::strategy::jitter)
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("bad configuration {0}")]
    BadConfiguration(#[from] ConfigurationError),

    #[error("failed to connect {0}")]
    ConnectionFailed(#[from] tonic::transport::Error),

    #[error("circuit breaker is open")]
    CircuitBreakerOpen,

    #[error("timeout has elapsed")]
    TimeoutElapsed,

    #[error("healthcheck client failed to connect {0}")]
    HealthCheckConnectFailed(#[from] HealthcheckError),

    #[error("rpc call failed with status {0}")]
    Status(#[from] Status),

    #[error("serialization or deserialization failed {0}")]
    SerDe(#[from] SerDeError),
}

impl From<std::convert::Infallible> for ClientError {
    fn from(_: std::convert::Infallible) -> Self {
        unimplemented!()
    }
}

impl From<client_executor::Error<ClientError>> for ClientError {
    fn from(e: client_executor::Error<ClientError>) -> Self {
        match e {
            client_executor::Error::Inner(e) => e,
            client_executor::Error::Rejected => Self::CircuitBreakerOpen,
            client_executor::Error::Elapsed => Self::TimeoutElapsed,
        }
    }
}

impl From<client_executor::Error<tonic::Status>> for ClientError {
    fn from(e: client_executor::Error<tonic::Status>) -> Self {
        match e {
            client_executor::Error::Inner(e) => Self::from(Status::from(e)),
            client_executor::Error::Rejected => Self::CircuitBreakerOpen,
            client_executor::Error::Elapsed => Self::TimeoutElapsed,
        }
    }
}

#[async_trait::async_trait]
pub trait Connectable: Sized {
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError>;
}

#[async_trait::async_trait]
pub trait Connect<C>: Sized
where
    C: Connectable + Clone,
{
    async fn connect(configuration: ClientConfiguration) -> Result<Self, ClientError>;

    async fn connect_with_healthcheck(
        configuration: ClientConfiguration,
        healthcheck_timeout: Duration,
        healthcheck_polling_interval: Duration,
    ) -> Result<Self, ClientError>;
}

pub(crate) mod client_impl {
    use std::{
        fmt::Debug,
        time::Duration,
    };

    use crate::graplinc::grapl::api::client::{
        Client,
        ClientConfiguration,
        ClientError,
        Connect,
        Connectable,
    };

    pub(crate) trait WithClient<C>
    where
        C: Connectable + Clone + Debug,
        Self: Sized,
    {
        const SERVICE_NAME: &'static str;

        fn with_client(client: Client<C>) -> Self;
    }

    #[async_trait::async_trait]
    impl<T, C> Connect<C> for T
    where
        T: WithClient<C> + Send,
        C: Connectable + Clone + Debug,
    {
        async fn connect(configuration: ClientConfiguration) -> Result<Self, ClientError> {
            let client = Client::connect(configuration).await?;
            Ok(Self::with_client(client))
        }

        async fn connect_with_healthcheck(
            configuration: ClientConfiguration,
            healthcheck_timeout: Duration,
            healthcheck_polling_interval: Duration,
        ) -> Result<Self, ClientError> {
            let client = Client::connect_with_healthcheck(
                configuration,
                Self::SERVICE_NAME,
                healthcheck_timeout,
                healthcheck_polling_interval,
            )
            .await?;

            Ok(Self::with_client(client))
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Client<C>
where
    C: Connectable + Clone + Debug,
{
    configuration: ClientConfiguration,
    proto_client: C,
    client_executor: Executor,
}

impl<C> Client<C>
where
    C: Connectable + Clone + Debug,
{
    pub(crate) async fn connect(configuration: ClientConfiguration) -> Result<Self, ClientError> {
        let connect_executor = Executor::new(ExecutorConfig::new(configuration.connect_timeout));
        let endpoint = configuration.endpoint()?;
        let proto_client = connect_executor
            .spawn(
                configuration
                    .connect_backoff_strategy()
                    .take(configuration.connect_retries),
                || {
                    let endpoint = endpoint.clone();
                    async move { C::connect(endpoint).await.map_err(ClientError::from) }
                },
            )
            .await?;

        let client_executor = Executor::new(ExecutorConfig::new(configuration.executor_timeout));

        Ok(Self {
            configuration,
            proto_client,
            client_executor,
        })
    }

    pub(crate) async fn connect_with_healthcheck(
        configuration: ClientConfiguration,
        service_name: &'static str,
        healthcheck_timeout: Duration,
        healthcheck_polling_interval: Duration,
    ) -> Result<Self, ClientError> {
        let endpoint = configuration.endpoint()?;
        HealthcheckClient::wait_until_healthy(
            endpoint,
            service_name,
            healthcheck_timeout,
            healthcheck_polling_interval,
        )
        .await?;

        Self::connect(configuration).await
    }

    pub(crate) async fn execute<PT, NT, PU, NU, P, F, R>(
        &self,
        request: NT,
        retry_predicate: P,
        max_retries: usize,
        grpc_call: F,
    ) -> Result<NU, ClientError>
    where
        PT: prost::Message + From<NT> + Clone,
        NT: ProtobufSerializable<ProtobufMessage = PT> + TryFrom<PT>,
        PU: prost::Message,
        NU: ProtobufSerializable<ProtobufMessage = PU> + TryFrom<PU>,
        P: Fn(&tonic::Status) -> bool,
        F: FnMut(C, tonic::Request<PT>) -> R + Clone,
        R: Future<Output = Result<tonic::Response<PU>, tonic::Status>>,
        ClientError: From<<NU as TryFrom<PU>>::Error>,
    {
        let proto_request = PT::try_from(request)?;
        let request_timeout = self.configuration.request_timeout;
        let client_executor = self.client_executor.clone();
        let backoff_strategy = self.configuration.backoff_strategy().take(max_retries);

        let result = client_executor
            .spawn_conditional(
                backoff_strategy,
                || {
                    let proto_client = self.proto_client.clone();
                    let proto_request = proto_request.clone();
                    let mut grpc_call = grpc_call.clone();

                    async move {
                        let mut tonic_request = tonic::Request::new(proto_request);
                        tonic_request.set_timeout(request_timeout);

                        grpc_call(proto_client, tonic_request).await
                    }
                    .in_current_span()
                },
                retry_predicate,
            )
            .await;

        match result {
            Ok(response) => {
                let response_proto = response.into_inner();
                Ok(NU::try_from(response_proto)?)
            }
            Err(e) => {
                let client_error = e.into();
                Err(client_error)
            }
        }
    }

    pub(crate) async fn execute_streaming<'a, S, PT, PU, NU, F, R>(
        &'a self,
        proto_stream: S,
        mut grpc_call: F,
    ) -> Result<NU, ClientError>
    where
        S: Stream<Item = PT> + 'static,
        PT: prost::Message + Clone + 'a,
        PU: prost::Message,
        NU: ProtobufSerializable<ProtobufMessage = PU> + TryFrom<PU>,
        F: FnMut(C, tonic::Request<S>) -> R + Clone,
        R: Future<Output = Result<tonic::Response<PU>, tonic::Status>>,
        ClientError: From<<NU as TryFrom<PU>>::Error>,
    {
        let request_timeout = self.configuration.request_timeout;
        let mut request = tonic::Request::new(proto_stream);
        request.set_timeout(request_timeout);

        let response_proto = grpc_call(self.proto_client.clone(), request)
            .in_current_span()
            .await
            .map_err(Status::from)?
            .into_inner();

        Ok(NU::try_from(response_proto)?)
    }
}
