//! This module contains the code to implement gRPC clients in
//! rust-proto. Specifically, you should use this code to automatically
//! implement much of the boilerplate required to translate between native
//! `ProtobufSerializable` types and generated `prost::Message` types in gRPC
//! API calls. Since Tonic doesn't give any interfaces to "hook onto", we use
//! the `Connectable` trait to impose such a thing on a client `C` generated by
//! Tonic. Then, we impose a constructor on our native gRPC client using
//! `WithClient<C>` which takes a `Client<C>`. Finally, within our native client
//! implementation we call `self.client.execute(...)` to implement the
//! "translated" gRPC API. This is easiest to see by example:
//!
//! ```ignore
//! use tonic::transport::Endpoint;
//!
//! use crate::{
//!     graplinc::grapl::api::{
//!         client::{
//!             Client,
//!             ClientError,
//!             Connectable,
//!             WithClient,
//!         },
//!         pipeline_ingress::v1beta1 as native,
//!     },
//!     protobufs::graplinc::grapl::api::pipeline_ingress::v1beta1::pipeline_ingress_service_client::PipelineIngressServiceClient,
//! };
//!
//! // First, implement Connectable for the client generated by Tonic.
//! #[async_trait::async_trait]
//! impl Connectable for PipelineIngressServiceClient<tonic::transport::Channel> {
//!     async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
//!         Ok(Self::connect(endpoint).await?)
//!     }
//! }
//!
//! // Next, implement native client which delegates to a Client<C>.
//! #[derive(Clone)]
//! pub struct PipelineIngressClient {
//!     client: Client<PipelineIngressServiceClient<tonic::transport::Channel>>,
//! }
//!
//! // Then implement WithClient<C> to give our native client a constructor
//! // in terms of C.
//! impl WithClient<PipelineIngressServiceClient<tonic::transport::Channel>> for PipelineIngressClient {
//!     fn with_client(
//!         client: Client<PipelineIngressServiceClient<tonic::transport::Channel>>,
//!     ) -> Self {
//!         Self { client }
//!     }
//! }
//!
//! // Finally, implement each of the gRPC methods and delegate to self.client
//! impl PipelineIngressClient {
//!     pub async fn publish_raw_log(
//!         &mut self,
//!         request: native::PublishRawLogRequest,
//!     ) -> Result<native::PublishRawLogResponse, ClientError> {
//!         self.client
//!             .execute(
//!                 request,
//!                 |status| status.code() == tonic::Code::Unavailable,
//!                 10,
//!                 |mut client, request| async move { client.publish_raw_log(request).await },
//!             )
//!             .await
//!     }
//! }
//! ```

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
use tonic::{transport::Endpoint,};
use tracing::Instrument;

use super::{protocol::status::Status, request_metadata::{InvalidRequestMetadataError, RequestMetadata}};
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

/// This struct contains all the configuration values necessary for connecting a
/// gRPC client to a gRPC server. We provide two methods of constructing the
/// configuration:
///
/// 1. The `new(...)` method -- this is probably only useful in unit tests where
/// you need to tightly control some mocked configuration data or don't want to
/// bother with the provider machinery.
///
/// 2. The `from<T>(provider: T)` method -- this is how you'll actually end up
/// creating a configuration in practice:
///
/// ``` no_run
/// use figment::{
///     Figment,
///     providers::Env,
/// };
///
/// let client_configuration = Figment::new()
///     .merge(Env::prefixed("MY_PREFIX_"))
///     .extract()?;
/// # Ok::<(), figment::Error>(())
/// ```
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientConfiguration {
    /// The network address for the gRPC server
    address: String,

    /// The max duration the request is allowed to take (gRPC timeout)
    #[serde(with = "humantime_serde")]
    request_timeout: Duration,

    /// Internal timeout for the request future
    #[serde(with = "humantime_serde")]
    executor_timeout: Duration,

    /// Max number of concurrent client connections
    concurrency_limit: usize,

    /// Duration to wait before retrying the first failed request -- the delay
    /// for the Nth failure is given by the formula:
    ///
    /// ```text
    /// backoff_delay = min(
    ///     initial_backoff_delay * fib(N),
    ///     maximum_backoff_delay
    /// )
    /// ```
    ///
    /// where `fib(N)` denotes the Nth Fibonacci number
    #[serde(with = "humantime_serde")]
    initial_backoff_delay: Duration,

    /// Max duration to wait between retries
    #[serde(with = "humantime_serde")]
    maximum_backoff_delay: Duration,

    /// The max duration allowed to establish a connection to the gRPC server
    #[serde(with = "humantime_serde")]
    connect_timeout: Duration,

    /// How many times to attempt connecting to the gRPC server before giving up
    connect_retries: usize,

    /// Duration to wait before trying to connect again -- the same Fibonacci
    /// backoff strategy is used to retry connection attempts as is used to
    /// retry requests
    #[serde(with = "humantime_serde")]
    connect_initial_backoff_delay: Duration,

    /// The max duration to wait between connection attempts
    #[serde(with = "humantime_serde")]
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

    #[error("rpc call failed with status {0}")]
    Status(#[from] Status),

    #[error("serialization or deserialization failed {0}")]
    SerDe(#[from] SerDeError),

    #[error("invalid metadata: {0}")]
    InvalidRequestMetadata(#[from] InvalidRequestMetadataError),
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

/// This trait is basically just a way to impose a constructor on `Self` which
/// takes a `Client<C>`. We use this to ensure that our public gRPC clients
/// exposed by rust-proto (e.g. `PipelineIngressClient`) wrap a particular
/// `Client<C>`.
pub(crate) trait WithClient<C>
where
    C: Connectable + Clone + Debug,
    Self: Sized,
{
    fn with_client(client: Client<C>) -> Self;
}

/// This trait is the "bridge" between the code generated by Tonic and our
/// applications. We impose this interface on the types generated by Tonic to
/// give our application-facing `Connect<C>` trait something to grab ahold of.
#[async_trait::async_trait]
pub trait Connectable: Sized {
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError>;
}

/// This trait is implemented (via a blanket implementation) for all our
/// application-facing client types. So you need only import your client of
/// choice (e.g. `PipelineIngressClient`) and this `Connect` trait, and you'll
/// have the ability to call `PipelineIngressClient::connect(config)`.
#[async_trait::async_trait]
pub trait Connect<C>: Sized
where
    C: Connectable + Clone,
{
    async fn connect(configuration: ClientConfiguration) -> Result<Self, ClientError>;
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
}

/// This thing is the go-between which translates Tonic's generated types to
/// application types and back. A public-facing client type will wrap this
/// object and delegate requests to it via the `execute` method.
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
    /// The blanket implementation of `Connect::connect` defined above delegates
    /// to this method. You should have no reason to call it "by hand" in this
    /// library. Instead, implement `Connectable` and `WithClient` and you'll
    /// get the blanket implementation of `Connect::connect` (and therefore
    /// you'll use this method) for free!
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

    /// Execute a gRPC request against an underlying gRPC client of type `C`
    /// generated by Tonic. This should be thought of as a function which takes
    /// a native `ProtobufSerializable` type `NT` to a native
    /// `ProtobufSerializable` type `NU` via intermediate `prost::Message` types
    /// `PT` and `PU`. The caller provides the code which actually executes the
    /// gRPC API call on the Tonic client `C` in the form of a `grpc_call`
    /// closure `F`.
    /// \
    /// # Params
    /// \
    /// - `request` -- The native `ProtobufSerializable` representation of a
    ///   gRPC request.
    ///
    /// - `retry_predicate` -- A predicate function which decides given the gRPC
    ///   response status whether to retry the request.
    ///
    /// - `max_retries` -- The maximum number of times to retry a failing
    ///   request. Retries will back off according to this instance's configured
    ///   `backoff_strategy`.
    ///
    /// - `grpc_call` -- A closure which executes the gRPC API call in terms of
    ///    types `PT` and `PU` on the underlying Tonic client of type `C`.
    /// \
    /// # Example usage
    /// \
    /// See [api/pipeline_ingress/v1beta1/client.rs](../api/pipeline_ingress/v1beta1/client.rs).
    /// \
    /// ```ignore
    /// impl PipelineIngressClient {
    ///     pub async fn publish_raw_log(
    ///         &mut self,
    ///         request: native::PublishRawLogRequest,
    ///     ) -> Result<native::PublishRawLogResponse, ClientError> {
    ///         self.client
    ///         .execute(
    ///             request,
    ///             None,
    ///             |status| status.code() == tonic::Code::Unavailable,
    ///             10,
    ///             |mut client, request| async move { client.publish_raw_log(request).await },
    ///         )
    ///         .await
    ///     }
    /// }
    /// ```
    pub(crate) async fn execute<PT, NT, PU, NU, P, F, R>(
        &self,
        request: NT,
        request_metadata: Option<RequestMetadata>,
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
        let request_metadata = request_metadata.map(|m| m.validate()).transpose()?;

        let result = client_executor
            .spawn_conditional(
                backoff_strategy,
                || {
                    let proto_client = self.proto_client.clone();
                    let proto_request = proto_request.clone();
                    let request_metadata = request_metadata.clone();
                    let mut grpc_call = grpc_call.clone();

                    async move {
                        let mut tonic_request = tonic::Request::new(proto_request);
                        tonic_request.set_timeout(request_timeout);
                        if let Some(request_metadata) = request_metadata {
                            request_metadata.merge_into(tonic_request.metadata_mut());
                        }

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

    /// Execute a client streaming gRPC request against an underlying gRPC
    /// client of type `C` generated by Tonic. This should be thought of as a
    /// function which takes a stream of `prost::Message` types `PT` to a native
    /// `ProtobufSerializable` type `NU` via the intermediate `prost::Message`
    /// type `PU`. The caller provides the code which actually executes the gRPC
    /// API call on the Tonic client `C` in the form of a `grpc_call` closure
    /// `F`.
    /// \
    /// # Params
    /// \
    /// - `request_timeout` -- The max duration the request is allowed to take
    ///    (gRPC timeout). Streaming requests can't be meaningfully
    ///    automatically retried, so they bypass the executor-based retry
    ///    machinery used for synchronous gRPC requests. Therefore we use a
    ///    per-request timeout here instead of the one specified in the
    ///    ClientConfiguration, and the caller is responsible for handling the
    ///    consequences of hitting this timeout.
    ///
    /// - `proto_stream` -- A stream of `prost::Message` types `PT`.
    ///
    /// - `grpc_call` -- A closure which executes the gRPC API call in terms of
    ///    types `S` and `PU` on the underlying Tonic client of type `C`.
    /// \
    /// # Example usage
    /// \
    /// See [api/plugin_registry/v1beta1_client.rs](../api/plugin_registry/v1beta1_client.rs).
    /// \
    /// ```ignore
    /// pub async fn create_plugin<S>(
    ///     &mut self,
    ///     metadata: native::PluginMetadata,
    ///     plugin_artifact: S,
    /// ) -> Result<native::CreatePluginResponse, ClientError>
    /// where
    ///     S: Stream<Item = Bytes> + Send + 'static,
    /// {
    ///     let proto_stream = futures::stream::iter(std::iter::once(
    ///         native::CreatePluginRequest::Metadata(metadata),
    ///     ))
    ///     .chain(plugin_artifact.map(native::CreatePluginRequest::Chunk))
    ///     .map(proto::CreatePluginRequest::from);
    ///
    ///     self.client
    ///         .execute_client_streaming(proto_stream, |mut client, request| async move {
    ///             client.create_plugin(request).await
    ///         })
    ///         .await
    /// }
    /// ```
    pub(crate) async fn execute_client_streaming<'a, S, PT, PU, NU, F, R>(
        &'a self,
        request_timeout: Duration,
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
