use std::{time::Duration, pin::Pin};

use futures::{
    channel::{oneshot::{
        self,
        Receiver,
        Sender,
    }, mpsc},
    Future,
    FutureExt, Stream, StreamExt, SinkExt,
};
use proto::plugin_registry_service_server::PluginRegistryService;
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
    execute_rpc,
    graplinc::grapl::api::plugin_registry::v1beta1::{
        CreatePluginRequestV2,
        CreatePluginResponseV2,
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
    protocol::{
        healthcheck::{
            server::init_health_service,
            HealthcheckError,
            HealthcheckStatus,
        },
        status::Status,
    },
    server_internals::GrpcApi,
    SerDeError,
};

// This complicated signature is suggested by Tonic:
// https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md#bidirectional-streaming-rpc

type CreatePluginResponseStream<E> = Pin<Box<dyn Stream<Item = Result<CreatePluginResponseV2, E>>>>;

/// Implement this trait to define the API business logic
#[tonic::async_trait]
pub trait PluginRegistryApi {
    type Error: Into<Status>;

    async fn create_plugin(
        &self,
        request: impl Stream<Item = CreatePluginRequestV2>,
    ) -> CreatePluginResponseStream<Self::Error>;

    async fn get_plugin(&self, request: GetPluginRequest)
        -> Result<GetPluginResponse, Self::Error>;

    async fn deploy_plugin(
        &self,
        request: DeployPluginRequest,
    ) -> Result<DeployPluginResponse, Self::Error>;

    async fn tear_down_plugin(
        &self,
        request: TearDownPluginRequest,
    ) -> Result<TearDownPluginResponse, Self::Error>;

    async fn get_generators_for_event_source(
        &self,
        request: GetGeneratorsForEventSourceRequest,
    ) -> Result<GetGeneratorsForEventSourceResponse, Self::Error>;

    async fn get_analyzers_for_tenant(
        &self,
        request: GetAnalyzersForTenantRequest,
    ) -> Result<GetAnalyzersForTenantResponse, Self::Error>;
}

//type ProtoCreatePluginResponseStream = Pin<Box<(dyn futures::Future<Output = Result<tonic::Response<proto::CreatePluginResponseV2>, tonic::Status>> + std::marker::Send)>>;


#[tonic::async_trait]
impl<T> PluginRegistryService for GrpcApi<T>
where
    T: PluginRegistryApi + Send + Sync + 'static,
{
    type CreatePluginStream = mpsc::Receiver<Result<proto::CreatePluginResponseV2, tonic::Status>>;
    async fn create_plugin(
        &self,
        request: Request<tonic::Streaming<proto::CreatePluginRequestV2>>,
    ) -> Result<Response<Self::CreatePluginStream>, tonic::Status>
    {
        // Based on
        // https://github.com/rkudryashov/exploring-rust-ecosystem/blob/master/grpc-telegram-bot/server/src/main.rs

        type Req = proto::CreatePluginRequestV2;
        type ProtoInner = Result<Req, tonic::Status>;
        type ProtoStream = dyn Stream<Item = ProtoInner>;
        let proto_stream = request.into_inner();

        let (
            mut input_tx,
            input_rx,
        ) = mpsc::channel(4);

        while let Some(proto_request_or_error) = proto_stream.next().await {
            let nativeized: CreatePluginRequestV2 = proto_request_or_error?.try_into()?;
            input_tx.send(nativeized);
        }

        let native_output_stream = self.api_server.create_plugin(input_rx).await;

        type NativeErr<T> = <T as PluginRegistryApi>::Error;
        let proto_output_stream = 
            native_output_stream.map(|result: Result<CreatePluginResponseV2, NativeErr<T>>| {
                result.map(Into::into).map_err(Into::into)
            })
        ;

        let (
            mut output_tx,
            output_rx,
        ) = mpsc::channel(4);

        proto_output_stream.send_all(output_tx);

        // fn takes: Stream<Native>, tx.send
        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(output_rx),
        )))
    }
    

    async fn get_plugin(
        &self,
        request: Request<proto::GetPluginRequest>,
    ) -> Result<Response<proto::GetPluginResponse>, tonic::Status> {
        execute_rpc!(self, request, get_plugin)
    }

    async fn deploy_plugin(
        &self,
        request: Request<proto::DeployPluginRequest>,
    ) -> Result<Response<proto::DeployPluginResponse>, tonic::Status> {
        execute_rpc!(self, request, deploy_plugin)
    }

    async fn tear_down_plugin(
        &self,
        request: Request<proto::TearDownPluginRequest>,
    ) -> Result<Response<proto::TearDownPluginResponse>, tonic::Status> {
        execute_rpc!(self, request, tear_down_plugin)
    }

    async fn get_generators_for_event_source(
        &self,
        request: Request<proto::GetGeneratorsForEventSourceRequest>,
    ) -> Result<Response<proto::GetGeneratorsForEventSourceResponse>, tonic::Status> {
        execute_rpc!(self, request, get_generators_for_event_source)
    }

    async fn get_analyzers_for_tenant(
        &self,
        request: Request<proto::GetAnalyzersForTenantRequest>,
    ) -> Result<Response<proto::GetAnalyzersForTenantResponse>, tonic::Status> {
        execute_rpc!(self, request, get_analyzers_for_tenant)
    }
}

/**
 * !!!!! IMPORTANT !!!!!
 * This is almost entirely cargo-culted from PipelineIngressServer.
 * Lots of opportunities to deduplicate and simplify.
 */
pub struct PluginRegistryServer<T, H, F>
where
    T: PluginRegistryApi + Send + Sync + 'static,
    H: Fn() -> F + Send + Sync + 'static,
    F: Future<Output = Result<HealthcheckStatus, HealthcheckError>> + Send + 'static,
{
    api_server: T,
    healthcheck: H,
    healthcheck_polling_interval: Duration,
    tcp_listener: TcpListener,
    shutdown_rx: Receiver<()>,
    service_name: &'static str,
}

impl<T, H, F> PluginRegistryServer<T, H, F>
where
    T: PluginRegistryApi + Send + Sync + 'static,
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
                service_name: PluginRegistryServiceProto::<GrpcApi<T>>::NAME,
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
        let (healthcheck_handle, health_service) =
            init_health_service::<PluginRegistryServiceProto<GrpcApi<T>>, _, _>(
                self.healthcheck,
                self.healthcheck_polling_interval,
            )
            .await;

        // TODO: add tower tracing, tls_config, concurrency limits
        Ok(Server::builder()
            .trace_fn(|request| {
                tracing::info_span!(
                    "Plugin Registry",
                    headers = ?request.headers(),
                    method = ?request.method(),
                    uri = %request.uri(),
                    extensions = ?request.extensions(),
                )
            })
            .add_service(health_service)
            .add_service(PluginRegistryServiceProto::new(GrpcApi::new(
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
