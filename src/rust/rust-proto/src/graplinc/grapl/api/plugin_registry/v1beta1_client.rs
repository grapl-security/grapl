use std::{
    convert::Infallible,
    fmt::Debug,
    time::Duration,
};

use bytes::Bytes;
use client_executor::{
    strategy::jitter,
    Executor,
    ExecutorConfig,
};
use futures::{
    Stream,
    StreamExt,
};
use proto::plugin_registry_service_client::PluginRegistryServiceClient as PluginRegistryServiceClientProto;

use crate::{
    execute_client_rpc,
    graplinc::grapl::api::plugin_registry::v1beta1 as native,
    protobufs::graplinc::grapl::api::plugin_registry::v1beta1 as proto,
    protocol::{
        endpoint::Endpoint,
        service_client::{
            ConnectError,
            Connectable,
        },
        status::Status,
    },
    SerDeError,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginRegistryServiceClientError {
    #[error("ErrorStatus {0}")]
    ErrorStatus(#[from] Status),
    #[error("PluginRegistryDeserializationError {0}")]
    PluginRegistryDeserializationError(#[from] SerDeError),
    #[error("CircuitOpen")]
    CircuitOpen,
    #[error("Timeout")]
    Elapsed,
}

// This likely should be removed in favor of making all SerDe conversions
// TryFrom<Error = SerDeError> instead of From.
// Currently it's kind of scattershot.
impl From<Infallible> for PluginRegistryServiceClientError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<client_executor::Error<tonic::Status>> for PluginRegistryServiceClientError {
    fn from(e: client_executor::Error<tonic::Status>) -> Self {
        match e {
            client_executor::Error::Inner(e) => Self::ErrorStatus(e.into()),
            client_executor::Error::Rejected => Self::CircuitOpen,
            client_executor::Error::Elapsed => Self::Elapsed,
        }
    }
}

#[derive(Clone)]
pub struct PluginRegistryServiceClient {
    proto_client: PluginRegistryServiceClientProto<tonic::transport::Channel>,
    executor: Executor,
}

#[async_trait::async_trait]
impl Connectable for PluginRegistryServiceClient {
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.plugin_registry.v1beta1.PluginRegistryService";

    #[tracing::instrument(err)]
    async fn connect(endpoint: Endpoint) -> Result<Self, ConnectError> {
        let executor = Executor::new(ExecutorConfig::new(Duration::from_secs(30)));
        let proto_client = executor
            .spawn(
                client_executor::strategy::FibonacciBackoff::from_millis(10)
                    .map(jitter)
                    .take(20),
                || {
                    let endpoint = endpoint.clone();
                    async move {
                        PluginRegistryServiceClientProto::connect(endpoint)
                            .await
                            .map_err(ConnectError::from)
                    }
                },
            )
            .await?;

        Ok(PluginRegistryServiceClient {
            proto_client,
            executor,
        })
    }
}

impl PluginRegistryServiceClient {
    /// create a new plugin.
    /// NOTE: Most consumers will want `create_plugin`, not `create_plugin_raw`.
    pub async fn create_plugin_raw<S>(
        &mut self,
        request: S,
    ) -> Result<native::CreatePluginResponse, PluginRegistryServiceClientError>
    where
        S: Stream<Item = native::CreatePluginRequest> + Send + 'static,
    {
        let response = match self
            .proto_client
            .create_plugin(request.map(proto::CreatePluginRequest::from))
            .await
        {
            Ok(r) => r,
            Err(e) => return Err(Status::from(e).into()),
        };
        let response = native::CreatePluginResponse::try_from(response.into_inner())?;
        Ok(response)
    }

    /// Create a new plugin
    ///
    pub async fn create_plugin<S>(
        &mut self,
        metadata: native::PluginMetadata,
        plugin_artifact: S,
    ) -> Result<native::CreatePluginResponse, PluginRegistryServiceClientError>
    where
        S: Stream<Item = Bytes> + Send + 'static,
    {
        // Send the metadata first followed by N chunks
        let request = futures::stream::iter(std::iter::once(
            native::CreatePluginRequest::Metadata(metadata),
        ))
        .chain(plugin_artifact.map(native::CreatePluginRequest::Chunk));

        self.create_plugin_raw(request).await
    }

    /// retrieve the plugin corresponding to the given plugin_id
    pub async fn get_plugin(
        &mut self,
        request: native::GetPluginRequest,
    ) -> Result<native::GetPluginResponse, PluginRegistryServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            get_plugin,
            proto::GetPluginRequest,
            native::GetPluginResponse,
        )
    }

    /// turn on a particular plugin's code
    pub async fn deploy_plugin(
        &mut self,
        request: native::DeployPluginRequest,
    ) -> Result<native::DeployPluginResponse, PluginRegistryServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            deploy_plugin,
            proto::DeployPluginRequest,
            native::DeployPluginResponse,
        )
    }

    /// turn off a particular plugin's code
    pub async fn tear_down_plugin(
        &mut self,
        request: native::TearDownPluginRequest,
    ) -> Result<native::TearDownPluginResponse, PluginRegistryServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            tear_down_plugin,
            proto::TearDownPluginRequest,
            native::TearDownPluginResponse,
        )
    }

    pub async fn get_plugin_health(
        &mut self,
        request: native::GetPluginHealthRequest,
    ) -> Result<native::GetPluginHealthResponse, PluginRegistryServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            get_plugin_health,
            proto::GetPluginHealthRequest,
            native::GetPluginHealthResponse,
        )
    }

    /// Given information about an event source, return all generators that handle that event source
    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_generators_for_event_source(
        &mut self,
        request: native::GetGeneratorsForEventSourceRequest,
    ) -> Result<native::GetGeneratorsForEventSourceResponse, PluginRegistryServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            get_generators_for_event_source,
            proto::GetGeneratorsForEventSourceRequest,
            native::GetGeneratorsForEventSourceResponse,
        )
    }

    /// Given information about a tenant, return all analyzers for that tenant
    pub async fn get_analyzers_for_tenant(
        &mut self,
        request: native::GetAnalyzersForTenantRequest,
    ) -> Result<native::GetAnalyzersForTenantResponse, PluginRegistryServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            get_analyzers_for_tenant,
            proto::GetAnalyzersForTenantRequest,
            native::GetAnalyzersForTenantResponse,
        )
    }
}
