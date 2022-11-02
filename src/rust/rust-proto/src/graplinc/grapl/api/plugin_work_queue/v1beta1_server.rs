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
use proto::plugin_work_queue_service_server::{
    PluginWorkQueueService,
    PluginWorkQueueServiceServer as ServerProto,
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
        plugin_work_queue::v1beta1 as native,
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
    protobufs::graplinc::grapl::api::plugin_work_queue::v1beta1 as proto,
};

/// Implement this trait to define the API business logic
#[tonic::async_trait]
pub trait PluginWorkQueueApi {
    type Error: Into<Status>;
    async fn push_execute_generator(
        &self,
        request: native::PushExecuteGeneratorRequest,
    ) -> Result<native::PushExecuteGeneratorResponse, Self::Error>;

    async fn push_execute_analyzer(
        &self,
        request: native::PushExecuteAnalyzerRequest,
    ) -> Result<native::PushExecuteAnalyzerResponse, Self::Error>;

    async fn get_execute_generator(
        &self,
        request: native::GetExecuteGeneratorRequest,
    ) -> Result<native::GetExecuteGeneratorResponse, Self::Error>;

    async fn get_execute_analyzer(
        &self,
        request: native::GetExecuteAnalyzerRequest,
    ) -> Result<native::GetExecuteAnalyzerResponse, Self::Error>;

    async fn acknowledge_generator(
        &self,
        request: native::AcknowledgeGeneratorRequest,
    ) -> Result<native::AcknowledgeGeneratorResponse, Self::Error>;

    async fn acknowledge_analyzer(
        &self,
        request: native::AcknowledgeAnalyzerRequest,
    ) -> Result<native::AcknowledgeAnalyzerResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T> PluginWorkQueueService for GrpcApi<T>
where
    T: PluginWorkQueueApi + Send + Sync + 'static,
{
    async fn push_execute_generator(
        &self,
        request: tonic::Request<proto::PushExecuteGeneratorRequest>,
    ) -> Result<tonic::Response<proto::PushExecuteGeneratorResponse>, tonic::Status> {
        execute_rpc!(self, request, push_execute_generator)
    }

    async fn push_execute_analyzer(
        &self,
        request: tonic::Request<proto::PushExecuteAnalyzerRequest>,
    ) -> Result<tonic::Response<proto::PushExecuteAnalyzerResponse>, tonic::Status> {
        execute_rpc!(self, request, push_execute_analyzer)
    }

    async fn get_execute_generator(
        &self,
        request: tonic::Request<proto::GetExecuteGeneratorRequest>,
    ) -> Result<tonic::Response<proto::GetExecuteGeneratorResponse>, tonic::Status> {
        execute_rpc!(self, request, get_execute_generator)
    }

    async fn get_execute_analyzer(
        &self,
        request: tonic::Request<proto::GetExecuteAnalyzerRequest>,
    ) -> Result<tonic::Response<proto::GetExecuteAnalyzerResponse>, tonic::Status> {
        execute_rpc!(self, request, get_execute_analyzer)
    }

    async fn acknowledge_generator(
        &self,
        request: tonic::Request<proto::AcknowledgeGeneratorRequest>,
    ) -> Result<tonic::Response<proto::AcknowledgeGeneratorResponse>, tonic::Status> {
        execute_rpc!(self, request, acknowledge_generator)
    }

    async fn acknowledge_analyzer(
        &self,
        request: tonic::Request<proto::AcknowledgeAnalyzerRequest>,
    ) -> Result<tonic::Response<proto::AcknowledgeAnalyzerResponse>, tonic::Status> {
        execute_rpc!(self, request, acknowledge_analyzer)
    }
}

/**
 * !!!!! IMPORTANT !!!!!
 * This is almost entirely cargo-culted from PipelineIngressServer.
 * Lots of opportunities to deduplicate and simplify.
 */
pub struct PluginWorkQueueServer<T, H, F>
where
    T: PluginWorkQueueApi + Send + Sync + 'static,
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

impl<T, H, F> PluginWorkQueueServer<T, H, F>
where
    T: PluginWorkQueueApi + Send + Sync + 'static,
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
                service_name: ServerProto::<GrpcApi<T>>::NAME,
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
            init_health_service::<ServerProto<GrpcApi<T>>, _, _>(
                self.healthcheck,
                self.healthcheck_polling_interval,
            )
            .await;

        // TODO: add tower tracing, tls_config, concurrency limits
        Ok(Server::builder()
            .add_service(health_service)
            .add_service(ServerProto::new(GrpcApi::new(self.api_server)))
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
