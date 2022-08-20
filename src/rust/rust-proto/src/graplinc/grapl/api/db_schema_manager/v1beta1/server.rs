#![allow(warnings)]
use std::time::Duration;

use futures::{
    channel::oneshot::{
        self,
        Receiver,
        Sender,
    },
    Future,
    FutureExt,
    SinkExt,
    StreamExt,
};
use proto::db_schema_manager_service_server::DbSchemaManagerService;
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
    graplinc::grapl::api::db_schema_manager::v1beta1::messages::{
        DeployGraphSchemasRequest,
        DeployGraphSchemasResponse,
    },
    protobufs::graplinc::grapl::api::db_schema_manager::v1beta1::{
        self as proto,
        db_schema_manager_service_server::DbSchemaManagerServiceServer as DbSchemaManagerServiceProto,
        DeployGraphSchemasRequest as DeployGraphSchemasRequestProto,
        DeployGraphSchemasResponse as DeployGraphSchemasResponseProto,
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
pub trait DbSchemaManagerApi {
    type Error: Into<Status>;
    async fn deploy_graph_schemas(
        &self,
        request: DeployGraphSchemasRequest,
    ) -> Result<DeployGraphSchemasResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T> DbSchemaManagerService for GrpcApi<T>
where
    T: DbSchemaManagerApi + Send + Sync + 'static,
{
    #[tracing::instrument(skip(self, request), err)]
    async fn deploy_graph_schemas(
        &self,
        request: Request<DeployGraphSchemasRequestProto>,
    ) -> Result<Response<DeployGraphSchemasResponseProto>, tonic::Status> {
        let proto_request = request.into_inner();

        let native_request = proto_request.try_into()?;

        let native_response = self
            .api_server
            .deploy_graph_schemas(native_request)
            .await
            .map_err(|e| e.into())?;

        let proto_response = native_response.try_into().map_err(SerDeError::from)?;

        Ok(Response::new(proto_response))
    }
}

/**
 * !!!!! IMPORTANT !!!!!
 * This is almost entirely cargo-culted from PipelineIngressServer.
 * Lots of opportunities to deduplicate and simplify.
 */
pub struct DbSchemaManagerServer<T, H, F>
where
    T: DbSchemaManagerApi + Send + Sync + 'static,
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

impl<T, H, F> DbSchemaManagerServer<T, H, F>
where
    T: DbSchemaManagerApi + Send + Sync + 'static,
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
                service_name: DbSchemaManagerServiceProto::<GrpcApi<T>>::NAME,
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
            init_health_service::<DbSchemaManagerServiceProto<GrpcApi<T>>, _, _>(
                self.healthcheck,
                self.healthcheck_polling_interval,
            )
            .await;

        // TODO: add tower tracing, tls_config, concurrency limits
        Ok(Server::builder()
            .trace_fn(|request| {
                tracing::info_span!(
                    "DbSchemaManager",
                    request_id = ?request.headers().get("x-request-id"),
                    method = ?request.method(),
                    uri = %request.uri(),
                    extensions = ?request.extensions(),
                )
            })
            .add_service(health_service)
            .add_service(DbSchemaManagerServiceProto::new(GrpcApi::new(
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
