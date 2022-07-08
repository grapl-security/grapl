use std::net::SocketAddr;

use futures::FutureExt;
use tokio::{
    net::TcpListener,
    sync::oneshot::Receiver,
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

use crate::{
    graplinc::grapl::api::graph_query_service::v1beta1::messages::{
        QueryGraphFromNodeRequest,
        QueryGraphFromNodeResponse,
        QueryGraphWithNodeRequest,
        QueryGraphWithNodeResponse,
    },
    protobufs::graplinc::grapl::api::graph_query_service::v1beta1::{
        graph_query_service_server::{
            GraphQueryService as GraphQueryServiceProto,
            GraphQueryServiceServer as GraphQueryServiceServerProto,
        },
        QueryGraphFromNodeRequest as QueryGraphFromNodeRequestProto,
        QueryGraphFromNodeResponse as QueryGraphFromNodeResponseProto,
        QueryGraphWithNodeRequest as QueryGraphWithNodeRequestProto,
        QueryGraphWithNodeResponse as QueryGraphWithNodeResponseProto,
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
        request: QueryGraphWithNodeRequest,
    ) -> Result<QueryGraphWithNodeResponse, Self::Error>;
    async fn query_graph_from_uid(
        &self,
        request: QueryGraphFromNodeRequest,
    ) -> Result<QueryGraphFromNodeResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T, E> GraphQueryServiceProto for T
where
    T: GraphQueryApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    async fn query_graph_with_uid(
        &self,
        request: tonic::Request<QueryGraphWithNodeRequestProto>,
    ) -> Result<tonic::Response<QueryGraphWithNodeResponseProto>, tonic::Status> {
        let request = request.into_inner();
        let request = request
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::invalid_argument(e.to_string()))?;
        let response = GraphQueryApi::query_graph_with_uid(self, request)
            .await
            .map_err(|e| e.into())?;

        Ok(tonic::Response::new(response.into()))
    }

    async fn query_graph_from_uid(
        &self,
        request: tonic::Request<QueryGraphFromNodeRequestProto>,
    ) -> Result<tonic::Response<QueryGraphFromNodeResponseProto>, tonic::Status> {
        let request = request.into_inner();
        let request = request
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::invalid_argument(e.to_string()))?;
        let response = GraphQueryApi::query_graph_from_uid(self, request)
            .await
            .map_err(|e| e.into())?;
        Ok(tonic::Response::new(response.into()))
    }
}

/// A server construct that drives the GraphQueryApi implementation.
pub struct GraphQueryServiceServer<T, E>
where
    T: GraphQueryApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    server: GraphQueryServiceServerProto<T>,
    addr: SocketAddr,
    shutdown_rx: Receiver<()>,
}

impl<T, E> GraphQueryServiceServer<T, E>
where
    T: GraphQueryApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    pub fn builder(
        service: T,
        addr: SocketAddr,
        shutdown_rx: Receiver<()>,
    ) -> GraphQueryServiceServerBuilder<T, E> {
        GraphQueryServiceServerBuilder::new(service, addr, shutdown_rx)
    }

    pub async fn serve(self) -> Result<(), GraphQueryServiceServerError> {
        let (healthcheck_handle, health_service) =
            init_health_service::<GraphQueryServiceServerProto<T>, _, _>(
                || async { Ok(HealthcheckStatus::Serving) },
                std::time::Duration::from_millis(500),
            )
            .await;

        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(GraphQueryServiceServerError::BindError)?;

        Server::builder()
            .trace_fn(|request| {
                tracing::trace_span!(
                    "GraphQuery",
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

pub struct GraphQueryServiceServerBuilder<T, E>
where
    T: GraphQueryApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    server: GraphQueryServiceServerProto<T>,
    addr: SocketAddr,
    shutdown_rx: Receiver<()>,
}

impl<T, E> GraphQueryServiceServerBuilder<T, E>
where
    T: GraphQueryApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    /// Create a new builder for a GraphQueryServiceServer,
    /// taking the required arguments upfront.
    pub fn new(service: T, addr: SocketAddr, shutdown_rx: Receiver<()>) -> Self {
        Self {
            server: GraphQueryServiceServerProto::new(service),
            addr,
            shutdown_rx,
        }
    }

    /// Consumes the builder and returns a new `GraphQueryServiceServer`.
    /// Note: Panics on invalid build state
    pub fn build(self) -> GraphQueryServiceServer<T, E> {
        GraphQueryServiceServer {
            server: self.server,
            addr: self.addr,
            shutdown_rx: self.shutdown_rx,
        }
    }
}
