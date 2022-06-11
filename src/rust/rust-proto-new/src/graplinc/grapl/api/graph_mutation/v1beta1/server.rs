
use std::net::SocketAddr;

use futures::FutureExt;
use tokio::{
    net::TcpListener,
    sync::oneshot::Receiver,
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    transport::Server,
};

use crate::{
    graplinc::grapl::api::graph_mutation::v1beta1::messages::{
        CreateEdgeRequest,
        CreateEdgeResponse,
        CreateNodeRequest,
        CreateNodeResponse,
        SetNodePropertyRequest,
        SetNodePropertyResponse,
    },
    protobufs::graplinc::grapl::api::graph_mutation::v1beta1::{
        graph_mutation_service_server::{
            GraphMutationService as GraphMutationServiceProto,
            GraphMutationServiceServer as GraphMutationServiceServerProto,
        },
        CreateEdgeRequest as CreateEdgeRequestProto,
        CreateEdgeResponse as CreateEdgeResponseProto,
        CreateNodeRequest as CreateNodeRequestProto,
        CreateNodeResponse as CreateNodeResponseProto,
        SetNodePropertyRequest as SetNodePropertyRequestProto,
        SetNodePropertyResponse as SetNodePropertyResponseProto,
    },

    protocol::{
        healthcheck::{
            server::init_health_service,
            HealthcheckStatus,
        },
        status::Status,
    },
    SerDeError,
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
        request: CreateNodeRequest,
    ) -> Result<CreateNodeResponse, Self::Error>;
    async fn set_node_property(
        &self,
        request: SetNodePropertyRequest,
    ) -> Result<SetNodePropertyResponse, Self::Error>;
    async fn create_edge(
        &self,
        request: CreateEdgeRequest,
    ) -> Result<CreateEdgeResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T, E> GraphMutationServiceProto for T
    where
        T: GraphMutationApi<Error=E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
{
    /// Create Node allocates a new node in the graph, returning the uid of the new node.
    async fn create_node(
        &self,
        request: tonic::Request<CreateNodeRequestProto>,
    ) -> Result<tonic::Response<CreateNodeResponseProto>, tonic::Status> {
        let request = request.into_inner();
        let request = request
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::invalid_argument(e.to_string()))?;
        let response = GraphMutationApi::create_node(self, request)
            .await
            .map_err(|e| e.into())?;

        Ok(tonic::Response::new(response.into()))
    }
    /// SetNodeProperty will update the property of the node with the given uid.
    /// If the node does not exist it will be created.
    async fn set_node_property(
        &self,
        request: tonic::Request<SetNodePropertyRequestProto>,
    ) -> Result<tonic::Response<SetNodePropertyResponseProto>, tonic::Status> {
        let request = request.into_inner();
        let request = request
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::invalid_argument(e.to_string()))?;
        let response = GraphMutationApi::set_node_property(self, request)
            .await
            .map_err(|e| e.into())?;

        Ok(tonic::Response::new(response.into()))
    }
    /// CreateEdge will create an edge with the name edge_name between the nodes
    /// that have the given uids. It will also create the reverse edge.
    async fn create_edge(
        &self,
        request: tonic::Request<CreateEdgeRequestProto>,
    ) -> Result<tonic::Response<CreateEdgeResponseProto>, tonic::Status> {
        let request = request.into_inner();
        let request: CreateEdgeRequest = request
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::invalid_argument(e.to_string()))?;
        let response = GraphMutationApi::create_edge(self, request)
            .await
            .map_err(|e| e.into())?;

        Ok(tonic::Response::new(response.into()))
    }
}

/// A server construct that drives the GraphMutationApi implementation.
pub struct GraphMutationServiceServer<T, E>
    where
        T: GraphMutationApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
{
    server: GraphMutationServiceServerProto<T>,
    addr: SocketAddr,
    shutdown_rx: Receiver<()>,
}

impl<T, E> GraphMutationServiceServer<T, E>
    where
        T: GraphMutationApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
{
    pub fn builder(
        service: T,
        addr: SocketAddr,
        shutdown_rx: Receiver<()>,
    ) -> GraphMutationServiceServerBuilder<T, E> {
        GraphMutationServiceServerBuilder::new(service, addr, shutdown_rx)
    }

    pub async fn serve(self) -> Result<(), GraphMutationServiceServerError> {
        let (healthcheck_handle, health_service) =
            init_health_service::<GraphMutationServiceServerProto<T>, _, _>(
                || async { Ok(HealthcheckStatus::Serving) },
                std::time::Duration::from_millis(500),
            )
                .await;

        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(GraphMutationServiceServerError::BindError)?;

        Server::builder()
            .trace_fn(|request| {
                tracing::trace_span!(
                        "GraphMutation",
                        headers = ?request.headers(),
                        method = ?request.method(),
                        uri = %request.uri(),
                        extensions = ?request.extensions(),
                    )
            })
            .add_service(health_service)
            .add_service(self.server)
            .serve_with_incoming_shutdown(
                TcpListenerStream::new(listener),
                self.shutdown_rx.map(|_| ()),
            )
            .then(|result| async move {
                healthcheck_handle.abort();
                result
            })
            .await?;
        Ok(())
    }
}

pub struct GraphMutationServiceServerBuilder<T, E>
    where
        T: GraphMutationApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
{
    server: GraphMutationServiceServerProto<T>,
    addr: SocketAddr,
    shutdown_rx: Receiver<()>,
}

impl<T, E> GraphMutationServiceServerBuilder<T, E>
    where
        T: GraphMutationApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
{
    /// Create a new builder for a GraphMutationServiceServer,
    /// taking the required arguments upfront.
    pub fn new(service: T, addr: SocketAddr, shutdown_rx: Receiver<()>) -> Self {
        Self {
            server: GraphMutationServiceServerProto::new(service),
            addr,
            shutdown_rx,
        }
    }

    /// Consumes the builder and returns a new `GraphMutationServiceServer`.
    /// Note: Panics on invalid build state
    pub fn build(self) -> GraphMutationServiceServer<T, E> {
        GraphMutationServiceServer {
            server: self.server,
            addr: self.addr,
            shutdown_rx: self.shutdown_rx,
        }
    }
}
