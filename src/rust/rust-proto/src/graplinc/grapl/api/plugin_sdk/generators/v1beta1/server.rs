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
};
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
    graplinc::grapl::api::plugin_sdk::generators::v1beta1 as native,
    protobufs::graplinc::grapl::api::plugin_sdk::generators::v1beta1::{
        self as proto,
        generator_service_server::{
            GeneratorService,
            GeneratorServiceServer as GeneratorServiceProto,
        },
    },
    protocol::{
        error::ServeError,
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

/// Implement this trait to define the API business logic
#[tonic::async_trait]
pub trait GeneratorApi {
    type Error: Into<Status>;

    async fn run_generator(
        &self,
        request: native::RunGeneratorRequest,
    ) -> Result<native::RunGeneratorResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T> GeneratorService for GrpcApi<T>
where
    T: GeneratorApi + Send + Sync + 'static,
{
    async fn run_generator(
        &self,
        request: Request<proto::RunGeneratorRequest>,
    ) -> Result<Response<proto::RunGeneratorResponse>, tonic::Status> {
        execute_rpc!(self, request, run_generator)
    }
}

/**
 * !!!!! IMPORTANT !!!!!
 * This is almost entirely cargo-culted from PipelineIngressServer.
 * Lots of opportunities to deduplicate and simplify.
 */
pub struct GeneratorServer<T, H, F>
where
    T: GeneratorApi + Send + Sync + 'static,
    H: Fn() -> F + Send + Sync + 'static,
    F: Future<Output = Result<HealthcheckStatus, HealthcheckError>> + Send + 'static,
{
    api_server: T,
    healthcheck: H,
    healthcheck_polling_interval: Duration,
    tcp_listener: TcpListener,
    shutdown_rx: Receiver<()>,
    service_name: &'static str,
    f_: PhantomData<F>,
}

impl<T, H, F> GeneratorServer<T, H, F>
where
    T: GeneratorApi + Send + Sync + 'static,
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
                service_name: GeneratorServiceProto::<GrpcApi<T>>::NAME,
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
    /// address. Returns a ServeError if the gRPC server cannot run.
    pub async fn serve(self) -> Result<(), ServeError> {
        let (healthcheck_handle, health_service) =
            init_health_service::<GeneratorServiceProto<GrpcApi<T>>, _, _>(
                self.healthcheck,
                self.healthcheck_polling_interval,
            )
            .await;

        // TODO: add tower tracing, concurrency limits
        let mut server_builder = Server::builder().trace_fn(|request| {
            tracing::info_span!(
                "exec_service",
                headers = ?request.headers(),
                method = ?request.method(),
                uri = %request.uri(),
                extensions = ?request.extensions(),
            )
        });

        Ok(server_builder
            .add_service(health_service)
            .add_service(GeneratorServiceProto::new(GrpcApi::new(self.api_server)))
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
