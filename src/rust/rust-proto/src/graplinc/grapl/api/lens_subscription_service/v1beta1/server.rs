use std::{
    net::SocketAddr,
    pin::Pin,
};

use futures::{
    FutureExt,
    Stream,
    StreamExt,
};
use tokio::{
    net::TcpListener,
    sync::oneshot::Receiver,
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    transport::Server,
    Response,
};

use crate::{
    graplinc::grapl::api::lens_subscription_service::v1beta1::messages::{
        SubscribeToLensRequest,
        SubscribeToLensResponse,
    },
    protobufs::graplinc::grapl::api::lens_subscription_service::v1beta1::{
        lens_subscription_service_server::{
            LensSubscriptionService as LensSubscriptionServiceProto,
            LensSubscriptionServiceServer as LensSubscriptionServiceServerProto,
        },
        SubscribeToLensRequest as SubscribeToLensRequestProto,
        SubscribeToLensResponse as SubscribeToLensResponseProto,
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

type ResponseStreamProto =
    Pin<Box<dyn Stream<Item = Result<SubscribeToLensResponseProto, tonic::Status>> + Send>>;
type SubscribeToLensResultProto<T> = Result<Response<T>, tonic::Status>;

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
    type SubscribeToLensStream: Stream<Item = Result<SubscribeToLensResponse, Self::Error>>
        + Send
        + 'static;
    async fn subscribe_to_lens(
        &self,
        request: SubscribeToLensRequest,
    ) -> Result<Self::SubscribeToLensStream, Self::Error>;
}

#[tonic::async_trait]
impl<T, E> LensSubscriptionServiceProto for T
where
    T: LensSubscriptionApi<Error = E> + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Status: From<E>,
{
    type SubscribeToLensStream = ResponseStreamProto;

    /// Create Node allocates a new node in the graph, returning the uid of the new node.
    async fn subscribe_to_lens(
        &self,
        request: tonic::Request<SubscribeToLensRequestProto>,
    ) -> SubscribeToLensResultProto<Self::SubscribeToLensStream> {
        let request = request.into_inner();
        let request = request
            .try_into()
            .map_err(|e: SerDeError| tonic::Status::invalid_argument(e.to_string()))?;
        let response = LensSubscriptionApi::subscribe_to_lens(self, request)
            .await
            .map_err(Status::from)
            .map_err(tonic::Status::from)?;
        let response = StreamExt::map(response, |response| {
            response
                .map(Into::into)
                .map_err(Status::from)
                .map_err(tonic::Status::from)
        });

        let s = Box::pin(response) as Self::SubscribeToLensStream;
        Ok(Response::new(s))
    }
}

/// A server construct that drives the LensSubscriptionApi implementation.
pub struct LensSubscriptionServiceServer<T, E>
where
    T: LensSubscriptionApi<Error = E> + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Status: From<E>,
{
    server: LensSubscriptionServiceServerProto<T>,
    addr: SocketAddr,
    shutdown_rx: Receiver<()>,
}

impl<T, E> LensSubscriptionServiceServer<T, E>
where
    T: LensSubscriptionApi<Error = E> + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Status: From<E>,
{
    pub fn builder(
        service: T,
        addr: SocketAddr,
        shutdown_rx: Receiver<()>,
    ) -> LensSubscriptionServiceServerBuilder<T, E> {
        LensSubscriptionServiceServerBuilder::new(service, addr, shutdown_rx)
    }

    pub async fn serve(self) -> Result<(), LensSubscriptionServiceServerError> {
        let (healthcheck_handle, health_service) =
            init_health_service::<LensSubscriptionServiceServerProto<T>, _, _>(
                || async { Ok(HealthcheckStatus::Serving) },
                std::time::Duration::from_millis(500),
            )
            .await;

        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(LensSubscriptionServiceServerError::BindError)?;

        Server::builder()
            .trace_fn(|request| {
                tracing::trace_span!(
                    "LensSubscription",
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

pub struct LensSubscriptionServiceServerBuilder<T, E>
where
    T: LensSubscriptionApi<Error = E> + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Status: From<E>,
{
    server: LensSubscriptionServiceServerProto<T>,
    addr: SocketAddr,
    shutdown_rx: Receiver<()>,
}

impl<T, E> LensSubscriptionServiceServerBuilder<T, E>
where
    T: LensSubscriptionApi<Error = E> + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Status: From<E>,
{
    /// Create a new builder for a LensSubscriptionServiceServer,
    /// taking the required arguments upfront.
    pub fn new(service: T, addr: SocketAddr, shutdown_rx: Receiver<()>) -> Self {
        Self {
            server: LensSubscriptionServiceServerProto::new(service),
            addr,
            shutdown_rx,
        }
    }

    /// Consumes the builder and returns a new `LensSubscriptionServiceServer`.
    /// Note: Panics on invalid build state
    pub fn build(self) -> LensSubscriptionServiceServer<T, E> {
        LensSubscriptionServiceServer {
            server: self.server,
            addr: self.addr,
            shutdown_rx: self.shutdown_rx,
        }
    }
}
