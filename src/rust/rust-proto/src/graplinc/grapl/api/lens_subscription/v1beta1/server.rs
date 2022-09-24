#![allow(warnings)]
use std::{
    marker::PhantomData,
    pin::Pin,
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
    Stream,
    StreamExt,
};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{
    NamedService,
    Server,
};

use crate::{
    graplinc::grapl::api::lens_subscription::v1beta1::messages::{
        SubscribeToLensRequest,
        SubscribeToLensResponse,
    },
    protobufs::graplinc::grapl::api::lens_subscription::v1beta1::{
        lens_subscription_service_server::{
            LensSubscriptionService as LensSubscriptionServiceProto,
            LensSubscriptionServiceServer as LensSubscriptionServiceServerProto,
        },
        SubscribeToLensRequest as SubscribeToLensRequestProto,
        SubscribeToLensResponse as SubscribeToLensResponseProto,
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

type ResponseStreamProto =
    Pin<Box<dyn Stream<Item = Result<SubscribeToLensResponseProto, tonic::Status>> + Send>>;
type SubscribeToLensResultProto<T> = Result<tonic::Response<T>, tonic::Status>;

#[derive(thiserror::Error, Debug)]
pub enum LensSubscriptionServiceServerError {
    #[error("grpc transport error: {0}")]
    GrpcTransportError(#[from] tonic::transport::Error),
    #[error("Bind error: {0}")]
    BindError(std::io::Error),
}

#[tonic::async_trait]
pub trait LensSubscriptionApi {
    type Error: Into<Status>;
    type SubscribeToLensStream: Stream<Item = Result<SubscribeToLensResponse, Self::Error>> + Send;

    async fn subscribe_to_lens(
        &self,
        request: SubscribeToLensRequest,
    ) -> Result<Self::SubscribeToLensStream, Self::Error>;
}

#[tonic::async_trait]
impl<T> LensSubscriptionServiceProto for GrpcApi<T>
where
    T: LensSubscriptionApi + Send + Sync + 'static,
{
    type SubscribeToLensStream = ResponseStreamProto;

    #[tracing::instrument(skip(self), err)]
    async fn subscribe_to_lens(
        &self,
        request: tonic::Request<SubscribeToLensRequestProto>,
    ) -> SubscribeToLensResultProto<Self::SubscribeToLensStream> {
        let request = request.into_inner();
        let request = request
            .try_into()
            .map_err(|e: crate::SerDeError| tonic::Status::invalid_argument(e.to_string()))?;
        let response = self
            .api_server
            .subscribe_to_lens(request)
            .await
            .map_err(Into::into)?;
        let response = StreamExt::map(response, move |response| {
            response
                .map(Into::into)
                .map_err(Into::into)
                .map_err(tonic::Status::from)
        });

        let s = Box::pin(response) as Self::SubscribeToLensStream;
        Ok(tonic::Response::new(s))
    }
}

/**
 * !!!!! IMPORTANT !!!!!
 * This is almost entirely cargo-culted from previous Server impls.
 * Lots of opportunities to deduplicate and simplify.
 */
/// A server construct that drives the LensSubscriptionApi implementation.
pub struct LensSubscriptionServiceServer<T, H, F>
where
    T: LensSubscriptionApi + Send + Sync + 'static,
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

impl<T, H, F> LensSubscriptionServiceServer<T, H, F>
where
    T: LensSubscriptionApi + Send + Sync + 'static,
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
                service_name: LensSubscriptionServiceServerProto::<GrpcApi<T>>::NAME,
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
            init_health_service::<LensSubscriptionServiceServerProto<GrpcApi<T>>, _, _>(
                self.healthcheck,
                self.healthcheck_polling_interval,
            )
            .await;

        // TODO: add tower tracing, concurrency limits
        let mut server_builder = Server::builder().trace_fn(|request| {
            tracing::info_span!(
                "LensSubscription",
                headers = ?request.headers(),
                method = ?request.method(),
                uri = %request.uri(),
                extensions = ?request.extensions(),
            )
        });

        Ok(server_builder
            .add_service(health_service)
            .add_service(LensSubscriptionServiceServerProto::new(GrpcApi::new(
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
