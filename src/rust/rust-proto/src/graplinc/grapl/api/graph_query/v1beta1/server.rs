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
use tonic::transport::{
    NamedService,
    Server,
};

use crate::{
    execute_rpc,
    graplinc::grapl::api::{
        graph_query::v1beta1::messages::{
            QueryGraphFromUidRequest,
            QueryGraphFromUidResponse,
            QueryGraphWithUidRequest,
            QueryGraphWithUidResponse,
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
        server::GrpcApi,
    },
    protobufs::graplinc::grapl::api::graph_query::v1beta1::{
        graph_query_service_server::{
            GraphQueryService as GraphQueryServiceProto,
            GraphQueryServiceServer as GraphQueryServiceServerProto,
        },
        QueryGraphFromUidRequest as QueryGraphFromUidRequestProto,
        QueryGraphFromUidResponse as QueryGraphFromUidResponseProto,
        QueryGraphWithUidRequest as QueryGraphWithUidRequestProto,
        QueryGraphWithUidResponse as QueryGraphWithUidResponseProto,
    },
};

#[derive(thiserror::Error, Debug)]
pub enum GraphQueryServiceServerError {
    #[error("grpc transport error: {0}")]
    GrpcTransportError(#[from] tonic::transport::Error),
    #[error("Bind error: {0}")]
    BindError(std::io::Error),
}

#[tonic::async_trait]
pub trait GraphQueryApi {
    type Error: Into<Status>;
    async fn query_graph_with_uid(
        &self,
        request: QueryGraphWithUidRequest,
    ) -> Result<QueryGraphWithUidResponse, Self::Error>;
    async fn query_graph_from_uid(
        &self,
        request: QueryGraphFromUidRequest,
    ) -> Result<QueryGraphFromUidResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T> GraphQueryServiceProto for GrpcApi<T>
where
    T: GraphQueryApi + Send + Sync + 'static,
{
    async fn query_graph_with_uid(
        &self,
        request: tonic::Request<QueryGraphWithUidRequestProto>,
    ) -> Result<tonic::Response<QueryGraphWithUidResponseProto>, tonic::Status> {
        execute_rpc!(self, request, query_graph_with_uid)
    }

    async fn query_graph_from_uid(
        &self,
        request: tonic::Request<QueryGraphFromUidRequestProto>,
    ) -> Result<tonic::Response<QueryGraphFromUidResponseProto>, tonic::Status> {
        execute_rpc!(self, request, query_graph_from_uid)
    }
}

/**
 * !!!!! IMPORTANT !!!!!
 * This is almost entirely cargo-culted from previous Server impls.
 * Lots of opportunities to deduplicate and simplify.
 */
/// A server construct that drives the GraphQueryApi implementation.
pub struct GraphQueryServiceServer<T, H, F>
where
    T: GraphQueryApi + Send + Sync + 'static,
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

impl<T, H, F> GraphQueryServiceServer<T, H, F>
where
    T: GraphQueryApi + Send + Sync + 'static,
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
                service_name: GraphQueryServiceServerProto::<GrpcApi<T>>::NAME,
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
            init_health_service::<GraphQueryServiceServerProto<GrpcApi<T>>, _, _>(
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
            .add_service(GraphQueryServiceServerProto::new(GrpcApi::new(
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
