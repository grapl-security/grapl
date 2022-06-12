use std::net::SocketAddr;

use futures::FutureExt;
use tokio::{
    net::TcpListener,
    sync::oneshot::Receiver,
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

use crate::{
    graplinc::grapl::api::schema_manager::v1beta1::messages::{
        DeployModelRequest,
        DeployModelResponse,
        GetEdgeSchemaRequest,
        GetEdgeSchemaResponse,
    },
    protobufs::graplinc::grapl::api::schema_manager::v1beta1::{
        schema_manager_service_server::{
            SchemaManagerService as SchemaManagerServiceProto,
            SchemaManagerServiceServer as SchemaManagerServiceServerProto,
        },
        DeployModelRequest as DeployModelRequestProto,
        DeployModelResponse as DeployModelResponseProto,
        GetEdgeSchemaRequest as GetEdgeSchemaRequestProto,
        GetEdgeSchemaResponse as GetEdgeSchemaResponseProto,
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
pub enum SchemaManagerServiceServerError {
    #[error("grpc transport error: {0}")]
    GrpcTransportError(#[from] tonic::transport::Error),
    #[error("Bind error: {0}")]
    BindError(std::io::Error),
}

#[tonic::async_trait]
pub trait SchemaManagerApi {
    type Error: Into<Status>;
    async fn deploy_model(
        &self,
        request: DeployModelRequest,
    ) -> Result<DeployModelResponse, Self::Error>;

    async fn get_edge_schema(
        &self,
        request: GetEdgeSchemaRequest,
    ) -> Result<GetEdgeSchemaResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T, E> SchemaManagerServiceProto for T
    where
        T: SchemaManagerApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
{
    /// Create Node allocates a new node in the graph, returning the uid of the new node.
    async fn deploy_model(
        &self,
        request: tonic::Request<DeployModelRequestProto>,
    ) -> Result<tonic::Response<DeployModelResponseProto>, tonic::Status> {
        let request = request.into_inner();
        let request = request
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::invalid_argument(e.to_string()))?;
        let response = SchemaManagerApi::deploy_model(self, request)
            .await
            .map_err(|e| e.into())?;

        Ok(tonic::Response::new(response.into()))
    }

    async fn get_edge_schema(
        &self,
        request: tonic::Request<GetEdgeSchemaRequestProto>,
    ) -> Result<tonic::Response<GetEdgeSchemaResponseProto>, tonic::Status> {
        let request = request.into_inner();
        let request = request
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::invalid_argument(e.to_string()))?;
        let response = SchemaManagerApi::get_edge_schema(self, request)
            .await
            .map_err(|e| e.into())?;

        Ok(tonic::Response::new(response.into()))
    }
}

/// A server construct that drives the SchemaManagerApi implementation.
pub struct SchemaManagerServiceServer<T, E>
    where
        T: SchemaManagerApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
{
    server: SchemaManagerServiceServerProto<T>,
    addr: SocketAddr,
    shutdown_rx: Receiver<()>,
}

impl<T, E> SchemaManagerServiceServer<T, E>
    where
        T: SchemaManagerApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
{
    pub fn builder(
        service: T,
        addr: SocketAddr,
        shutdown_rx: Receiver<()>,
    ) -> SchemaManagerServiceServerBuilder<T, E> {
        SchemaManagerServiceServerBuilder::new(service, addr, shutdown_rx)
    }

    pub async fn serve(self) -> Result<(), SchemaManagerServiceServerError> {
        let (healthcheck_handle, health_service) =
            init_health_service::<SchemaManagerServiceServerProto<T>, _, _>(
                || async { Ok(HealthcheckStatus::Serving) },
                std::time::Duration::from_millis(500),
            )
                .await;

        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(SchemaManagerServiceServerError::BindError)?;

        Server::builder()
            .trace_fn(|request| {
                tracing::trace_span!(
                    "SchemaManager",
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

pub struct SchemaManagerServiceServerBuilder<T, E>
    where
        T: SchemaManagerApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
{
    server: SchemaManagerServiceServerProto<T>,
    addr: SocketAddr,
    shutdown_rx: Receiver<()>,
}

impl<T, E> SchemaManagerServiceServerBuilder<T, E>
    where
        T: SchemaManagerApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
{
    /// Create a new builder for a SchemaManagerServiceServer,
    /// taking the required arguments upfront.
    pub fn new(service: T, addr: SocketAddr, shutdown_rx: Receiver<()>) -> Self {
        Self {
            server: SchemaManagerServiceServerProto::new(service),
            addr,
            shutdown_rx,
        }
    }

    /// Consumes the builder and returns a new `SchemaManagerServiceServer`.
    /// Note: Panics on invalid build state
    pub fn build(self) -> SchemaManagerServiceServer<T, E> {
        SchemaManagerServiceServer {
            server: self.server,
            addr: self.addr,
            shutdown_rx: self.shutdown_rx,
        }
    }
}
