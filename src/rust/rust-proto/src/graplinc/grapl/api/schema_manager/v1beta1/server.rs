use std::{
    marker::PhantomData,
    time::Duration,
};

use futures::{
    Future,
    FutureExt,
};
use tokio::{
    net::TcpListener,
    sync::oneshot,
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{
    NamedService,
    Server,
};

use crate::{
    execute_rpc,
    graplinc::grapl::api::schema_manager::v1beta1::messages::{
        DeploySchemaRequest,
        DeploySchemaResponse,
        GetEdgeSchemaRequest,
        GetEdgeSchemaResponse,
    },
    protobufs::graplinc::grapl::api::schema_manager::{
        v1beta1 as proto,
        v1beta1::schema_manager_service_server::{
            SchemaManagerService as SchemaManagerServiceProto,
            SchemaManagerServiceServer as SchemaManagerServiceServerProto,
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

#[derive(thiserror::Error, Debug)]
pub enum SchemaManagerServiceServerError {
    #[error("grpc transport error: {0}")]
    GrpcTransportError(#[from] tonic::transport::Error),
    #[error("Bind error: {0}")]
    BindError(std::io::Error),
}

#[tonic::async_trait]
pub trait SchemaManagerApi {
    type Error: Into<Status>;
    async fn deploy_schema(
        &self,
        request: DeploySchemaRequest,
    ) -> Result<DeploySchemaResponse, Self::Error>;

    async fn get_edge_schema(
        &self,
        request: GetEdgeSchemaRequest,
    ) -> Result<GetEdgeSchemaResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T> SchemaManagerServiceProto for GrpcApi<T>
where
    T: SchemaManagerApi + Send + Sync + 'static,
{
    async fn deploy_schema(
        &self,
        request: tonic::Request<proto::DeploySchemaRequest>,
    ) -> Result<tonic::Response<proto::DeploySchemaResponse>, tonic::Status> {
        execute_rpc!(self, request, deploy_schema)
    }

    async fn get_edge_schema(
        &self,
        request: tonic::Request<proto::GetEdgeSchemaRequest>,
    ) -> Result<tonic::Response<proto::GetEdgeSchemaResponse>, tonic::Status> {
        execute_rpc!(self, request, get_edge_schema)
    }
}

/**
 * !!!!! IMPORTANT !!!!!
 * This is almost entirely cargo-culted from previous Server impls.
 * Lots of opportunities to deduplicate and simplify.
 */
pub struct SchemaManagerServer<T, H, F>
where
    T: SchemaManagerApi + Send + Sync + 'static,
    H: Fn() -> F + Send + Sync + 'static,
    F: Future<Output = Result<HealthcheckStatus, HealthcheckError>> + Send + 'static,
{
    api_server: T,
    healthcheck: H,
    healthcheck_polling_interval: Duration,
    tcp_listener: TcpListener,
    shutdown_rx: oneshot::Receiver<()>,
    service_name: &'static str,
    f_: PhantomData<F>,
}

impl<T, H, F> SchemaManagerServer<T, H, F>
where
    T: SchemaManagerApi + Send + Sync + 'static,
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
    ) -> (Self, oneshot::Sender<()>) {
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        (
            Self {
                api_server,
                healthcheck,
                healthcheck_polling_interval,
                tcp_listener,
                shutdown_rx,
                service_name: SchemaManagerServiceServerProto::<GrpcApi<T>>::NAME,
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
            init_health_service::<SchemaManagerServiceServerProto<GrpcApi<T>>, _, _>(
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
            .add_service(SchemaManagerServiceServerProto::new(GrpcApi::new(
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
