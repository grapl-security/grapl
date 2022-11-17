#![allow(dead_code)] // Will be removed once wimax adds the service in immediate next PR

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
    graplinc::grapl::api::{
        event_source::v1beta1 as native,
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
    protobufs::graplinc::grapl::api::event_source::v1beta1::{
        self as proto,
        event_source_service_server::{
            EventSourceService,
            EventSourceServiceServer as ServerProto,
        },
    },
};

/// Implement this trait to define the API business logic
#[tonic::async_trait]
pub trait EventSourceApi {
    type Error: Into<Status>;

    async fn create_event_source(
        &self,
        request: native::CreateEventSourceRequest,
    ) -> Result<native::CreateEventSourceResponse, Self::Error>;

    async fn update_event_source(
        &self,
        request: native::UpdateEventSourceRequest,
    ) -> Result<native::UpdateEventSourceResponse, Self::Error>;

    async fn get_event_source(
        &self,
        request: native::GetEventSourceRequest,
    ) -> Result<native::GetEventSourceResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T> EventSourceService for GrpcApi<T>
where
    T: EventSourceApi + Send + Sync + 'static,
{
    async fn create_event_source(
        &self,
        request: Request<proto::CreateEventSourceRequest>,
    ) -> Result<Response<proto::CreateEventSourceResponse>, tonic::Status> {
        execute_rpc!(self, request, create_event_source)
    }

    async fn update_event_source(
        &self,
        request: Request<proto::UpdateEventSourceRequest>,
    ) -> Result<Response<proto::UpdateEventSourceResponse>, tonic::Status> {
        execute_rpc!(self, request, update_event_source)
    }

    async fn get_event_source(
        &self,
        request: Request<proto::GetEventSourceRequest>,
    ) -> Result<Response<proto::GetEventSourceResponse>, tonic::Status> {
        execute_rpc!(self, request, get_event_source)
    }
}

/**
 * !!!!! IMPORTANT !!!!!
 * This is almost entirely cargo-culted from previous Server impls.
 * Lots of opportunities to deduplicate and simplify.
 */
pub struct EventSourceServer<T, H, F>
where
    T: EventSourceApi + Send + Sync + 'static,
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

impl<T, H, F> EventSourceServer<T, H, F>
where
    T: EventSourceApi + Send + Sync + 'static,
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
            init_health_service::<ServerProto<GrpcApi<T>>, _, _>(
                self.healthcheck,
                self.healthcheck_polling_interval,
            )
            .await;

        // TODO: add tower tracing, concurrency limits
        let mut server_builder = Server::builder().trace_fn(crate::server_tracing::server_trace_fn);

        Ok(server_builder
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
