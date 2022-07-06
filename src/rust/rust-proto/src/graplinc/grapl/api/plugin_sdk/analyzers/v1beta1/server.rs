use std::net::SocketAddr;

use futures::FutureExt;
use tokio::{
    net::TcpListener,
    sync::oneshot::Receiver,
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

use crate::{
    graplinc::grapl::api::plugin_sdk::analyzers::v1beta1::messages::{
        RunAnalyzerRequest,
        RunAnalyzerResponse,
    },
    protobufs::graplinc::grapl::api::plugin_sdk::analyzers::v1beta1::{
        analyzer_service_server::{
            AnalyzerService as AnalyzerServiceProto,
            AnalyzerServiceServer as AnalyzerServiceServerProto,
        },
        RunAnalyzerRequest as RunAnalyzerRequestProto,
        RunAnalyzerResponse as RunAnalyzerResponseProto,
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
pub enum AnalyzerServiceServerError {
    #[error("grpc transport error: {0}")]
    GrpcTransportError(#[from] tonic::transport::Error),
    #[error("Bind error: {0}")]
    BindError(std::io::Error),
}

#[tonic::async_trait]
pub trait AnalyzerApi {
    type Error: Into<Status>;
    async fn run_analyzer(
        &self,
        request: RunAnalyzerRequest,
    ) -> Result<RunAnalyzerResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T, E> AnalyzerServiceProto for T
    where
        T: AnalyzerApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
{
    async fn run_analyzer(
        &self,
        request: tonic::Request<RunAnalyzerRequestProto>,
    ) -> Result<tonic::Response<RunAnalyzerResponseProto>, tonic::Status> {
        let request = request.into_inner();
        let request = request
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::invalid_argument(e.to_string()))?;
        let response = AnalyzerApi::run_analyzer(self, request)
            .await
            .map_err(|e| e.into())?;

        Ok(tonic::Response::new(response.into()))
    }

}

/// A server construct that drives the AnalyzerApi implementation.
pub struct AnalyzerServiceServer<T, E>
    where
        T: AnalyzerApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
{
    server: AnalyzerServiceServerProto<T>,
    addr: SocketAddr,
    shutdown_rx: Receiver<()>,
}

impl<T, E> AnalyzerServiceServer<T, E>
    where
        T: AnalyzerApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
{
    pub fn builder(
        service: T,
        addr: SocketAddr,
        shutdown_rx: Receiver<()>,
    ) -> AnalyzerServiceServerBuilder<T, E> {
        AnalyzerServiceServerBuilder::new(service, addr, shutdown_rx)
    }

    pub async fn serve(self) -> Result<(), AnalyzerServiceServerError> {
        let (healthcheck_handle, health_service) =
            init_health_service::<AnalyzerServiceServerProto<T>, _, _>(
                || async { Ok(HealthcheckStatus::Serving) },
                std::time::Duration::from_millis(500),
            )
                .await;

        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(AnalyzerServiceServerError::BindError)?;

        Server::builder()
            .trace_fn(|request| {
                tracing::trace_span!(
                    "Analyzer",
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

pub struct AnalyzerServiceServerBuilder<T, E>
    where
        T: AnalyzerApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
{
    server: AnalyzerServiceServerProto<T>,
    addr: SocketAddr,
    shutdown_rx: Receiver<()>,
}

impl<T, E> AnalyzerServiceServerBuilder<T, E>
    where
        T: AnalyzerApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
{
    /// Create a new builder for a AnalyzerServiceServer,
    /// taking the required arguments upfront.
    pub fn new(service: T, addr: SocketAddr, shutdown_rx: Receiver<()>) -> Self {
        Self {
            server: AnalyzerServiceServerProto::new(service),
            addr,
            shutdown_rx,
        }
    }

    /// Consumes the builder and returns a new `AnalyzerServiceServer`.
    /// Note: Panics on invalid build state
    pub fn build(self) -> AnalyzerServiceServer<T, E> {
        AnalyzerServiceServer {
            server: self.server,
            addr: self.addr,
            shutdown_rx: self.shutdown_rx,
        }
    }
}
