use std::{
    convert::TryInto,
    time::Duration,
};

use client_executor::{
    Executor,
    ExecutorConfig,
};
use futures::{Future, Stream, StreamExt};
use thiserror::Error;
use tonic::transport::Endpoint;
use tracing::Instrument;

use crate::SerDe;

use super::protocol::{
    healthcheck::client::HealthcheckClient,
    status::Status
};

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("endpoint is not properly formatted {0}")]
    BadEndpoint(#[from] tonic::transport::Error),
}

#[derive(Clone)]
pub struct Configuration<B>
where
    B: IntoIterator<Item = Duration> + Clone
{
    endpoint: Endpoint,
    service_name: &'static str,
    backoff_strategy: B,
    executor_timeout: Duration,
    request_timeout: Duration,
}

impl <B> Configuration<B>
where
    B: IntoIterator<Item = Duration> + Clone
{
    pub fn new<A>(
        service_name: &'static str,
        address: A,
        request_timeout: Duration,
        executor_timeout: Duration,
        concurrency_limit: usize,
        backoff_strategy: B,
    ) -> Result<Self, ConfigurationError>
    where
        A: TryInto<Endpoint>,
        ConfigurationError: From<<A as TryInto<Endpoint>>::Error>,
    {
        let endpoint: Endpoint = address.try_into()?
            .concurrency_limit(concurrency_limit)
            .timeout(request_timeout);

        Ok(Self {
            endpoint,
            service_name,
            backoff_strategy,
            executor_timeout,
            request_timeout,
        })
    }

    pub fn backoff_strategy(&self) -> impl Iterator<Item = Duration> {
        self.backoff_strategy.clone().into_iter()
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("failed to connect {0}")]
    ConnectError(#[from] tonic::transport::Error),

    #[error("attempted to connect a client which was already connected")]
    AlreadyConnected,

    #[error("attempted to use a client which has not yet been connected")]
    NotYetConnected,

    #[error("circuit breaker is open")]
    CircuitBreakerOpen,

    #[error("timeout has elapsed")]
    TimeoutElapsed,

    #[error("healthcheck client failed to connect {0}")]
    HealthCheckConnectFailed(#[from] super::protocol::healthcheck::HealthcheckError),

    #[error("rpc call failed with status {0}")]
    Status(#[from] Status)
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
pub(crate) trait Connectable
{
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError>;
}

pub struct Client<B, C>
where
    B: IntoIterator<Item = Duration> + Clone,
    C: Connectable + Clone,
{
    configuration: Configuration<B>,
    proto_client: Option<C>,
    client_executor: Executor,
}

impl <B, C> Client<B, C>
where
    B: IntoIterator<Item = Duration> + Clone,
    C: Connectable + Clone,
{
    pub fn new(configuration: Configuration<B>) -> Self {
        let executor_timeout = configuration.executor_timeout;
        Self {
            configuration,
            proto_client: None,
            client_executor: Executor::new(ExecutorConfig::new(executor_timeout)),
        }
    }

    pub async fn connect<S>(
        self,
        connect_timeout: Duration,
        connect_retries: usize,
        connect_backoff_strategy: S
    ) -> Result<Self, ClientError>
    where
        S: Iterator<Item = Duration>,
    {
        if self.proto_client.is_some() {
            return Err(ClientError::AlreadyConnected)
        }

        let executor = Executor::new(ExecutorConfig::new(connect_timeout));
        let proto_client = executor
            .spawn(
                connect_backoff_strategy.take(connect_retries),
                || {
                    let endpoint = self.configuration.endpoint.clone();
                    async move {
                        C::connect(endpoint)
                            .await
                            .map_err(ClientError::from)
                    }
                })
            .await?;

        Ok(Self {
            configuration: self.configuration,
            proto_client: Some(proto_client),
            client_executor: self.client_executor,
        })
    }

    pub async fn connect_with_healthcheck<S>(
        self,
        healthcheck_timeout: Duration,
        healthcheck_polling_interval: Duration,
        connect_timeout: Duration,
        connect_retries: usize,
        connect_backoff_strategy: S,
    ) -> Result<Self, ClientError>
    where
        S: Iterator<Item = Duration>,
    {
        HealthcheckClient::wait_until_healthy(
            self.configuration.endpoint.clone(),
            self.configuration.service_name,
            healthcheck_timeout,
            healthcheck_polling_interval,
        )
            .await?;

        self.connect(
            connect_timeout,
            connect_retries,
            connect_backoff_strategy,
        )
        .await
    }

    pub(crate) async fn execute<PT, NT, PU, NU, P, F, R>(
        &self,
        request: NT,
        retry_predicate: P,
        max_retries: usize,
        grpc_call: F,
    ) -> Result<NU, ClientError>
    where
        PT: prost::Message,
        NT: SerDe + From<PT>,
        PU: prost::Message,
        NU: SerDe + From<PU>,
        P: Fn(&tonic::Status) -> bool,
        F: FnMut(C, tonic::Request<PT>) -> R + Clone,
        R: Future<Output = Result<tonic::Response<PU>, tonic::Status>>
    {
        if let Some(proto_client) = &self.proto_client {
            let proto_request = PT::try_from(request)?;
            let request_timeout = self.configuration.request_timeout;
            let client_executor = self.client_executor.clone();
            let backoff_strategy = self.configuration
                .backoff_strategy()
                .take(max_retries);

            let result = client_executor.spawn_conditional(
                backoff_strategy,
                || {
                    let proto_client = proto_client.clone();
                    let proto_request = proto_request.clone();
                    let mut grpc_call = grpc_call.clone();

                    async move {
                        let mut tonic_request = tonic::Request::new(proto_request);
                        tonic_request.set_timeout(request_timeout);

                        grpc_call(proto_client, tonic_request).await
                    }.in_current_span()
                },
                retry_predicate,
            ).await;

            match result {
                Ok(response) => {
                    let response_proto = response.into_inner();
                    Ok(NU::try_from(response_proto)?)
                },
                Err(e) => {
                    let client_error = e.into();
                    Err(client_error)
                },
            }
        } else {
            Err(ClientError::NotYetConnected)
        }
    }

    pub(crate) async fn execute_streaming<PS, NS, PT, NT, PU, NU, P, F, R>(
        &self,
        request: NS,
        retry_predicate: P,
        max_retries: usize,
        grpc_call: F,
    ) -> Result<NU, ClientError>
    where
        PS: Stream<Item = PT> + Send + 'static,
        NS: Stream<Item = NT> + Send + 'static,
        PT: prost::Message,
        NT: SerDe + From<PT>,
        PU: prost::Message,
        NU: SerDe + From<PU>,
        P: Fn(&tonic::Status) -> bool,
        F: FnMut(C, tonic::Request<PS>) -> R + Clone,
        R: Future<Output = Result<tonic::Response<PU>, tonic::Status>>
    {
        if let Some(proto_client) = &self.proto_client {
            let proto_stream: PS = request.map(|req| PT::from(req));
            let request_timeout = self.configuration.request_timeout;
            let client_executor = self.client_executor.clone();
            let backoff_strategy = self.configuration
                .backoff_strategy()
                .take(max_retries);

            let result = client_executor.spawn_conditional(
                backoff_strategy,
                || {
                    let proto_client = proto_client.clone();
                    let proto_stream = proto_stream.clone();
                    let mut grpc_call = grpc_call.clone();

                    async move {
                        let mut tonic_request = tonic::Request::new(proto_stream);
                        tonic_request.set_timeout(request_timeout);

                        grpc_call(proto_client, tonic_request).await
                    }.in_current_span()
                },
                retry_predicate,
            ).await;

            match result {
                Ok(response) => {
                    let response_proto = response.into_inner();
                    Ok(NU::try_from(response_proto)?)
                },
                Err(e) => {
                    let client_error = e.into();
                    Err(client_error)
                },
            }
        } else {
            Err(ClientError::NotYetConnected)
        }
    }
}
