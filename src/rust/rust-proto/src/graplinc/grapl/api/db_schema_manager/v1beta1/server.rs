use std::net::SocketAddr;

use futures::FutureExt;
use tokio::{
    net::TcpListener,
    sync::oneshot::Receiver,
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

use crate::{
    graplinc::grapl::api::db_schema_manager::v1beta1::messages::{
        DeployGraphSchemasRequest,
        DeployGraphSchemasResponse,
    },
    protobufs::graplinc::grapl::api::db_schema_manager::v1beta1::{
        db_schema_manager_service_server::{
            DbSchemaManagerService as DbSchemaManagerServiceProto,
            DbSchemaManagerServiceServer as DbSchemaManagerServiceServerProto,
        },
        DeployGraphSchemasRequest as DeployGraphSchemasRequestProto,
        DeployGraphSchemasResponse as DeployGraphSchemasResponseProto,
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
pub enum DbSchemaManagerServiceServerError {
    #[error("grpc transport error: {0}")]
    GrpcTransportError(#[from] tonic::transport::Error),
    #[error("Bind error: {0}")]
    BindError(std::io::Error),
}

#[tonic::async_trait]
pub trait DbSchemaManagerApi {
    type Error: Into<Status>;
    async fn deploy_graph_schemas(
        &self,
        request: DeployGraphSchemasRequest,
    ) -> Result<DeployGraphSchemasResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T, E> DbSchemaManagerServiceProto for T
where
    T: DbSchemaManagerApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    async fn deploy_graph_schemas(
        &self,
        request: tonic::Request<DeployGraphSchemasRequestProto>,
    ) -> Result<tonic::Response<DeployGraphSchemasResponseProto>, tonic::Status> {
        let request = request.into_inner();
        let request = request
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::invalid_argument(e.to_string()))?;
        let response = DbSchemaManagerApi::deploy_graph_schemas(self, request)
            .await
            .map_err(|e| e.into())?;

        Ok(tonic::Response::new(response.into()))
    }
}

/// A server construct that drives the DbSchemaManagerApi implementation.
pub struct DbSchemaManagerServiceServer<T, E>
where
    T: DbSchemaManagerApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    server: DbSchemaManagerServiceServerProto<T>,
    addr: SocketAddr,
    shutdown_rx: Receiver<()>,
}

impl<T, E> DbSchemaManagerServiceServer<T, E>
where
    T: DbSchemaManagerApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    pub fn builder(
        service: T,
        addr: SocketAddr,
        shutdown_rx: Receiver<()>,
    ) -> DbSchemaManagerServiceServerBuilder<T, E> {
        DbSchemaManagerServiceServerBuilder::new(service, addr, shutdown_rx)
    }

    pub async fn serve(self) -> Result<(), DbSchemaManagerServiceServerError> {
        let (healthcheck_handle, health_service) =
            init_health_service::<DbSchemaManagerServiceServerProto<T>, _, _>(
                || async { Ok(HealthcheckStatus::Serving) },
                std::time::Duration::from_millis(500),
            )
            .await;

        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(DbSchemaManagerServiceServerError::BindError)?;

        Server::builder()
            .trace_fn(|request| {
                tracing::trace_span!(
                    "DbSchemaManager",
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

pub struct DbSchemaManagerServiceServerBuilder<T, E>
where
    T: DbSchemaManagerApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    server: DbSchemaManagerServiceServerProto<T>,
    addr: SocketAddr,
    shutdown_rx: Receiver<()>,
}

impl<T, E> DbSchemaManagerServiceServerBuilder<T, E>
where
    T: DbSchemaManagerApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    /// Create a new builder for a DbSchemaManagerServiceServer,
    /// taking the required arguments upfront.
    pub fn new(service: T, addr: SocketAddr, shutdown_rx: Receiver<()>) -> Self {
        Self {
            server: DbSchemaManagerServiceServerProto::new(service),
            addr,
            shutdown_rx,
        }
    }

    /// Consumes the builder and returns a new `DbSchemaManagerServiceServer`.
    /// Note: Panics on invalid build state
    pub fn build(self) -> DbSchemaManagerServiceServer<T, E> {
        DbSchemaManagerServiceServer {
            server: self.server,
            addr: self.addr,
            shutdown_rx: self.shutdown_rx,
        }
    }
}
