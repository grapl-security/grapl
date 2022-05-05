use std::{
    marker::PhantomData,
    time::Duration,
};

use futures::{
    channel::oneshot::{
        self,
        Receiver,
        Sender,
    },
    Future,
    FutureExt,
    TryFutureExt,
};
use proto::plugin_registry_service_server::PluginRegistryService;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    transport::{
        NamedService,
        Server,
    },
    Request,
    Response,
};

use crate::{
    graplinc::grapl::api::plugin_registry::v1beta1::{
        CreatePluginRequest,
        CreatePluginResponse,
        DeployPluginRequest,
        DeployPluginResponse,
        GetAnalyzersForTenantRequest,
        GetAnalyzersForTenantResponse,
        GetGeneratorsForEventSourceRequest,
        GetGeneratorsForEventSourceResponse,
        GetPluginRequest,
        GetPluginResponse,
        TearDownPluginRequest,
        TearDownPluginResponse,
    },
    protobufs::graplinc::grapl::api::plugin_registry::v1beta1::{
        self as proto,
        plugin_registry_service_server::PluginRegistryServiceServer as PluginRegistryServiceProto,
    },
    protocol::status::Status,
    server_internals::ServerInternalGrpc,
    SerDeError,
};

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum PluginRegistryApiError {
    #[error("failed to serialize/deserialize {0}")]
    SerDeError(#[from] SerDeError),

    #[error("received unfavorable gRPC status {0}")]
    GrpcStatus(#[from] tonic::Status),
}

/// Implement this trait to define the API business logic
#[tonic::async_trait]
pub trait PluginRegistryApi<E>
where
    E: Into<Status>,
{
    async fn create_plugin(&self, request: CreatePluginRequest) -> Result<CreatePluginResponse, E>;

    async fn get_plugin(&self, request: GetPluginRequest) -> Result<GetPluginResponse, E>;

    async fn deploy_plugin(&self, request: DeployPluginRequest) -> Result<DeployPluginResponse, E>;

    async fn tear_down_plugin(
        &self,
        request: TearDownPluginRequest,
    ) -> Result<TearDownPluginResponse, E>;

    async fn get_generators_for_event_source(
        &self,
        request: GetGeneratorsForEventSourceRequest,
    ) -> Result<GetGeneratorsForEventSourceResponse, E>;

    async fn get_analyzers_for_tenant(
        &self,
        request: GetAnalyzersForTenantRequest,
    ) -> Result<GetAnalyzersForTenantResponse, E>;
}

#[tonic::async_trait]
impl<T, E> PluginRegistryService for ServerInternalGrpc<T, E>
where
    T: PluginRegistryApi<E> + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Status: From<E>,
{
    async fn create_plugin(
        &self,
        request: Request<proto::CreatePluginRequest>,
    ) -> Result<Response<proto::CreatePluginResponse>, tonic::Status> {
        let inner_request: CreatePluginRequest = request
            .into_inner()
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::unknown(e.to_string()))?;

        let response = self
            .api_server
            .create_plugin(inner_request)
            .map_err(Status::from)
            .await?;

        Ok(Response::new(response.into()))
    }

    async fn get_plugin(
        &self,
        request: Request<proto::GetPluginRequest>,
    ) -> Result<Response<proto::GetPluginResponse>, tonic::Status> {
        let inner_request: GetPluginRequest = request
            .into_inner()
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::unknown(e.to_string()))?;

        let response = self
            .api_server
            .get_plugin(inner_request)
            .map_err(Status::from)
            .await?;

        Ok(Response::new(response.into()))
    }

    async fn deploy_plugin(
        &self,
        request: Request<proto::DeployPluginRequest>,
    ) -> Result<Response<proto::DeployPluginResponse>, tonic::Status> {
        let inner_request: DeployPluginRequest = request
            .into_inner()
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::unknown(e.to_string()))?;

        let response = self
            .api_server
            .deploy_plugin(inner_request)
            .map_err(Status::from)
            .await?;

        Ok(Response::new(response.into()))
    }

    async fn tear_down_plugin(
        &self,
        _request: Request<proto::TearDownPluginRequest>,
    ) -> Result<Response<proto::TearDownPluginResponse>, tonic::Status> {
        todo!()
    }

    #[tracing::instrument(skip(self, _request), err)]
    async fn get_generators_for_event_source(
        &self,
        _request: Request<proto::GetGeneratorsForEventSourceRequest>,
    ) -> Result<Response<proto::GetGeneratorsForEventSourceResponse>, tonic::Status> {
        todo!()
    }

    async fn get_analyzers_for_tenant(
        &self,
        _request: Request<proto::GetAnalyzersForTenantRequest>,
    ) -> Result<Response<proto::GetAnalyzersForTenantResponse>, tonic::Status> {
        todo!()
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum HealthcheckError {
    #[error("not found {0}")]
    NotFound(String),

    #[error("healthcheck failed {0}")]
    HealthcheckFailed(String),
}

#[non_exhaustive]
#[derive(Debug)]
pub enum HealthcheckStatus {
    Serving,
    NotServing,
    Unknown,
}

/**
 * !!!!! IMPORTANT !!!!!
 * This is almost entirely cargo-culted from PipelineIngressServer.
 * Lots of opportunities to deduplicate and simplify.
 */
pub struct PluginRegistryServer<T, E, H, F>
where
    T: PluginRegistryApi<E> + Send + Sync + 'static,
    E: Into<Status>,
    H: Fn() -> F + Send + Sync + 'static,
    F: Future<Output = Result<HealthcheckStatus, HealthcheckError>> + Send,
{
    api_server: T,
    healthcheck: H,
    healthcheck_polling_interval: Duration,
    tcp_listener: TcpListener,
    shutdown_rx: Receiver<()>,
    service_name: &'static str,
    e_: PhantomData<E>,
    f_: PhantomData<F>,
}

impl<T, E, H, F> PluginRegistryServer<T, E, H, F>
where
    T: PluginRegistryApi<E> + Send + Sync + 'static,
    E: Sync + Send + 'static,
    Status: From<E>,
    H: Fn() -> F + Send + Sync + 'static,
    F: Future<Output = Result<HealthcheckStatus, HealthcheckError>> + Send,
{
    /// Construct a new gRPC server which will serve the given API
    /// implementation on the given socket address. Server is constructed in
    /// a non-running state. Call the serve() method to run the server. This
    /// method also returns a channel you can use to trigger server
    /// shutdown.
    pub fn new(
        api_server: T,
        tcp_listener: TcpListener,
        healthcheck: H,
        healthcheck_polling_interval: Duration,
    ) -> (Self, Sender<()>) {
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        (
            Self {
                api_server,
                healthcheck,
                healthcheck_polling_interval,
                tcp_listener,
                shutdown_rx,
                service_name: PluginRegistryServiceProto::<ServerInternalGrpc<T, E>>::NAME,
                e_: PhantomData,
                f_: PhantomData,
            },
            shutdown_tx,
        )
    }

    /// returns the service name associated with this service. You will need
    /// this value to construct a HealthcheckClient with which to query this
    /// service's healthcheck.
    pub fn service_name(&self) -> &'static str {
        self.service_name
    }

    /// Run the gRPC server and serve the API on this server's socket
    /// address. Returns a ConfigurationError if the gRPC server cannot run.
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

        // we configure our health reporter initially in the not_serving
        // state s.t. clients which are waiting for this service to start
        // can wait for the state change to the serving state
        health_reporter
            .set_not_serving::<PluginRegistryServiceProto<ServerInternalGrpc<T, E>>>()
            .await;

        let healthcheck_handle = tokio::task::spawn(async move {
            loop {
                match (self.healthcheck)().await {
                    Ok(status) => match status {
                        HealthcheckStatus::Serving => {
                            tracing::info!("healthcheck status \"serving\"");
                            health_reporter
                                    .set_serving::<PluginRegistryServiceProto<
                                        ServerInternalGrpc<T, E>,
                                    >>()
                                    .await
                        }
                        HealthcheckStatus::NotServing => {
                            tracing::warn!("healthcheck status \"not serving\"");
                            health_reporter
                                    .set_not_serving::<PluginRegistryServiceProto<
                                        ServerInternalGrpc<T, E>,
                                    >>()
                                    .await
                        }
                        HealthcheckStatus::Unknown => {
                            tracing::warn!("healthcheck status \"unknown\"");
                            health_reporter
                                    .set_not_serving::<PluginRegistryServiceProto<
                                        ServerInternalGrpc<T, E>,
                                    >>()
                                    .await
                        }
                    },
                    Err(e) => {
                        // healthcheck failed, so we'll set_not_serving()
                        tracing::error!("healthcheck error {}", e);
                        health_reporter
                            .set_not_serving::<PluginRegistryServiceProto<ServerInternalGrpc<T, E>>>()
                            .await
                    }
                }

                tokio::time::sleep(self.healthcheck_polling_interval).await;
            }
        });

        // TODO: add tower tracing, tls_config, concurrency limits
        Ok(Server::builder()
            .add_service(health_service)
            .add_service(PluginRegistryServiceProto::new(ServerInternalGrpc::new(
                self.api_server,
            )))
            .serve_with_incoming_shutdown(
                TcpListenerStream::new(self.tcp_listener),
                self.shutdown_rx.map(|_| ()),
            )
            .then(|result| async move {
                healthcheck_handle.abort();
                result
            })
            .await?)
    }
}
