use std::time::Duration;

use bytes::Bytes;
use client_executor::{
    Executor,
    ExecutorConfig,
};
use futures::{
    Stream,
    StreamExt,
};
use proto::plugin_registry_service_client::PluginRegistryServiceClient as PluginRegistryServiceClientProto;
use tracing::instrument;

use crate::{
    create_proto_client,
    execute_client_rpc,
    graplinc::grapl::api::{
        client_macros::RpcConfig,
        plugin_registry::v1beta1 as native,
        protocol::{
            endpoint::Endpoint,
            error::GrpcClientError,
            service_client::{
                ConnectError,
                Connectable,
            },
            status::Status,
        },
    },
    protobufs::graplinc::grapl::api::plugin_registry::v1beta1 as proto,
};

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
        let proto_client = create_proto_client!(
            executor,
            PluginRegistryServiceClientProto<tonic::transport::Channel>,
            endpoint,
        );

        Ok(PluginRegistryServiceClient {
            proto_client,
            executor,
        })
    }
}

impl PluginRegistryServiceClient {
    /// create a new plugin.
    /// NOTE: Most consumers will want `create_plugin`, not `create_plugin_raw`.
    /// NOTE: streaming RPCs aren't hooked up to client-executor just yet.
    #[instrument(skip(self, request), err)]
    pub async fn create_plugin_raw<S>(
        &mut self,
        request: S,
    ) -> Result<native::CreatePluginResponse, GrpcClientError>
    where
        S: Stream<Item = native::CreatePluginRequest> + Send + 'static,
    {
        let proto_response = self
            .proto_client
            .create_plugin(request.map(proto::CreatePluginRequest::from))
            .await
            .map_err(Status::from)?;
        let native_response = native::CreatePluginResponse::try_from(proto_response.into_inner())?;
        Ok(native_response)
    }

    /// Create a new plugin
    ///
    #[instrument(skip(self, metadata, plugin_artifact), err)]
    pub async fn create_plugin<S>(
        &mut self,
        metadata: native::PluginMetadata,
        plugin_artifact: S,
    ) -> Result<native::CreatePluginResponse, GrpcClientError>
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
    #[instrument(skip(self, request), err)]
    pub async fn get_plugin(
        &mut self,
        request: native::GetPluginRequest,
    ) -> Result<native::GetPluginResponse, GrpcClientError> {
        execute_client_rpc!(
            self,
            request,
            get_plugin,
            proto::GetPluginRequest,
            native::GetPluginResponse,
            RpcConfig::default(),
        )
    }

    #[instrument(skip(self, request), err)]
    pub async fn list_plugins(
        &mut self,
        request: native::ListPluginsRequest,
    ) -> Result<native::ListPluginsResponse, GrpcClientError> {
        execute_client_rpc!(
            self,
            request,
            list_plugins,
            proto::ListPluginsRequest,
            native::ListPluginsResponse,
            RpcConfig::default(),
        )
    }

    #[instrument(skip(self, request), err)]
    pub async fn get_plugin_deployment(
        &mut self,
        request: native::GetPluginDeploymentRequest,
    ) -> Result<native::GetPluginDeploymentResponse, GrpcClientError> {
        execute_client_rpc!(
            self,
            request,
            get_plugin_deployment,
            proto::GetPluginDeploymentRequest,
            native::GetPluginDeploymentResponse,
            RpcConfig::default(),
        )
    }

    /// turn on a particular plugin's code
    #[instrument(skip(self, request), err)]
    pub async fn deploy_plugin(
        &mut self,
        request: native::DeployPluginRequest,
    ) -> Result<native::DeployPluginResponse, GrpcClientError> {
        execute_client_rpc!(
            self,
            request,
            deploy_plugin,
            proto::DeployPluginRequest,
            native::DeployPluginResponse,
            RpcConfig::default(),
        )
    }

    /// turn off a particular plugin's code
    #[instrument(skip(self, request), err)]
    pub async fn tear_down_plugin(
        &mut self,
        request: native::TearDownPluginRequest,
    ) -> Result<native::TearDownPluginResponse, GrpcClientError> {
        execute_client_rpc!(
            self,
            request,
            tear_down_plugin,
            proto::TearDownPluginRequest,
            native::TearDownPluginResponse,
            RpcConfig::default(),
        )
    }

    #[instrument(skip(self, request), err)]
    pub async fn get_plugin_health(
        &mut self,
        request: native::GetPluginHealthRequest,
    ) -> Result<native::GetPluginHealthResponse, GrpcClientError> {
        execute_client_rpc!(
            self,
            request,
            get_plugin_health,
            proto::GetPluginHealthRequest,
            native::GetPluginHealthResponse,
            RpcConfig::default(),
        )
    }

    /// Given information about an event source, return all generators that handle that event source
    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_generators_for_event_source(
        &mut self,
        request: native::GetGeneratorsForEventSourceRequest,
    ) -> Result<native::GetGeneratorsForEventSourceResponse, GrpcClientError> {
        execute_client_rpc!(
            self,
            request,
            get_generators_for_event_source,
            proto::GetGeneratorsForEventSourceRequest,
            native::GetGeneratorsForEventSourceResponse,
            RpcConfig::default(),
        )
    }

    /// Given information about a tenant, return all analyzers for that tenant
    #[instrument(skip(self, request), err)]
    pub async fn get_analyzers_for_tenant(
        &mut self,
        request: native::GetAnalyzersForTenantRequest,
    ) -> Result<native::GetAnalyzersForTenantResponse, GrpcClientError> {
        execute_client_rpc!(
            self,
            request,
            get_analyzers_for_tenant,
            proto::GetAnalyzersForTenantRequest,
            native::GetAnalyzersForTenantResponse,
            RpcConfig::default(),
        )
    }
}
