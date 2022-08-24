use std::time::Duration;

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
    graplinc::grapl::api::graph_mutation::v1beta1::messages as native,
    protobufs::graplinc::grapl::api::graph_mutation::v1beta1::{
        self as proto,
        graph_mutation_service_server::{
            GraphMutationService as GraphMutationServiceProto,
            GraphMutationServiceServer as GraphMutationServiceServerProto,
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
};

#[derive(thiserror::Error, Debug)]
pub enum GraphMutationServiceServerError {
    #[error("grpc transport error: {0}")]
    GrpcTransportError(#[from] tonic::transport::Error),
    #[error("Bind error: {0}")]
    BindError(std::io::Error),
}

#[tonic::async_trait]
pub trait GraphMutationApi {
    type Error: Into<Status>;
    async fn create_node(
        &self,
        request: native::CreateNodeRequest,
    ) -> Result<native::CreateNodeResponse, Self::Error>;
    async fn set_node_property(
        &self,
        request: native::SetNodePropertyRequest,
    ) -> Result<native::SetNodePropertyResponse, Self::Error>;
    async fn create_edge(
        &self,
        request: native::CreateEdgeRequest,
    ) -> Result<native::CreateEdgeResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T> GraphMutationServiceProto for GrpcApi<T>
where
    T: GraphMutationApi + Send + Sync + 'static,
{
    /// Create Node allocates a new node in the graph, returning the uid of the new node.
    async fn create_node(
        &self,
        request: tonic::Request<proto::CreateNodeRequest>,
    ) -> Result<tonic::Response<proto::CreateNodeResponse>, tonic::Status> {
        execute_rpc!(self, request, create_node)
    }
    /// SetNodeProperty will update the property of the node with the given uid.
    /// If the node does not exist it will be created.
    async fn set_node_property(
        &self,
        request: tonic::Request<proto::SetNodePropertyRequest>,
    ) -> Result<tonic::Response<proto::SetNodePropertyResponse>, tonic::Status> {
        execute_rpc!(self, request, set_node_property)
    }
    /// CreateEdge will create an edge with the name edge_name between the nodes
    /// that have the given uids. It will also create the reverse edge.
    async fn create_edge(
        &self,
        request: tonic::Request<proto::CreateEdgeRequest>,
    ) -> Result<tonic::Response<proto::CreateEdgeResponse>, tonic::Status> {
        execute_rpc!(self, request, create_edge)
    }
}

/**
 * !!!!! IMPORTANT !!!!!
 * This is almost entirely cargo-culted from PipelineIngressServer.
 * Lots of opportunities to deduplicate and simplify.
 */
pub struct GraphMutationServer<T, H, F>
where
    T: GraphMutationApi + Send + Sync + 'static,
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

impl<T, H, F> GraphMutationServer<T, H, F>
where
    T: GraphMutationApi + Send + Sync + 'static,
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
                service_name: GraphMutationServiceServerProto::<GrpcApi<T>>::NAME,
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
            init_health_service::<GraphMutationServiceServerProto<GrpcApi<T>>, _, _>(
                self.healthcheck,
                self.healthcheck_polling_interval,
            )
            .await;

        // TODO: add tower tracing, tls_config, concurrency limits
        Ok(Server::builder()
            .add_service(health_service)
            .add_service(GraphMutationServiceServerProto::new(GrpcApi::new(
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
