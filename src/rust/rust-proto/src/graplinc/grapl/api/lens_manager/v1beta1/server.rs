
use std::net::SocketAddr;

use futures::FutureExt;
use tokio::{
    net::TcpListener,
    sync::oneshot::Receiver,
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    transport::Server,
    Request,
    Response,
};

use crate::{
    graplinc::grapl::api::lens_manager::v1beta1::messages::{
        AddNodeToScopeRequest,
        AddNodeToScopeResponse,
        CloseLensRequest,
        CloseLensResponse,
        CreateLensRequest,
        CreateLensResponse,
        MergeLensRequest,
        MergeLensResponse,
        RemoveNodeFromAllScopesRequest,
        RemoveNodeFromAllScopesResponse,
        RemoveNodeFromScopeRequest,
        RemoveNodeFromScopeResponse,
    },
    protobufs::graplinc::grapl::api::lens_manager::v1beta1::{
        lens_manager_service_server::{
            LensManagerService as LensManagerServiceProto,
            LensManagerServiceServer as LensManagerServiceServerProto,
        },
        AddNodeToScopeRequest as AddNodeToScopeRequestProto,
        AddNodeToScopeResponse as AddNodeToScopeResponseProto,
        CloseLensRequest as CloseLensRequestProto,
        CloseLensResponse as CloseLensResponseProto,
        CreateLensRequest as CreateLensRequestProto,
        CreateLensResponse as CreateLensResponseProto,
        MergeLensRequest as MergeLensRequestProto,
        MergeLensResponse as MergeLensResponseProto,
        RemoveNodeFromAllScopesRequest as RemoveNodeFromAllScopesRequestProto,
        RemoveNodeFromAllScopesResponse as RemoveNodeFromAllScopesResponseProto,
        RemoveNodeFromScopeRequest as RemoveNodeFromScopeRequestProto,
        RemoveNodeFromScopeResponse as RemoveNodeFromScopeResponseProto,
    },
    protocol::{
        healthcheck::{
            server::init_health_service,
            HealthcheckStatus,
        },
        status::Status,
    },
};

#[tonic::async_trait]
pub trait LensManagerApi {
    type Error: Into<Status>;

    /// Creates a new lens with an empty scope
    async fn create_lens(&self, request: CreateLensRequest) -> Result<CreateLensResponse, Self::Error>;
    /// MergeLens adds the scope of one lens to another
    async fn merge_lens(&self, request: MergeLensRequest) -> Result<MergeLensResponse, Self::Error>;
    /// CloseLens will remove a Lens node from the graph, detaching it from its scope
    async fn close_lens(&self, request: CloseLensRequest) -> Result<CloseLensResponse, Self::Error>;
    /// Adds a given entity node to the scope of a lens
    async fn add_node_to_scope(
        &self,
        request: AddNodeToScopeRequest,
    ) -> Result<AddNodeToScopeResponse, Self::Error>;
    /// Remove a node from a given lens's scope
    async fn remove_node_from_scope(
        &self,
        request: RemoveNodeFromScopeRequest,
    ) -> Result<RemoveNodeFromScopeResponse, Self::Error>;
    /// Remove a node from all of the lens scopes it is attached to
    async fn remove_node_from_all_scopes(
        &self,
        request: RemoveNodeFromAllScopesRequest,
    ) -> Result<RemoveNodeFromAllScopesResponse, Self::Error>;
}

#[tonic::async_trait]
impl<T, E> LensManagerServiceProto for T
where
    T: LensManagerApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    /// Creates a new lens with an empty scope
    async fn create_lens(
        &self,
        raw_request: Request<CreateLensRequestProto>,
    ) -> Result<Response<CreateLensResponseProto>, tonic::Status> {
        let proto_request = raw_request.into_inner();
        let request: CreateLensRequest = match proto_request.try_into() {
            Ok(request) => request,
            Err(e) => return Err(tonic::Status::invalid_argument(e.to_string())),
        };
        let response = LensManagerApi::create_lens(self, request.into()).await
            .map_err(|e| e.into())?;

        Ok(Response::new(response.into()))
    }
    /// MergeLens adds the scope of one lens to another
    async fn merge_lens(
        &self,
        raw_request: Request<MergeLensRequestProto>,
    ) -> Result<Response<MergeLensResponseProto>, tonic::Status> {
        let proto_request = raw_request.into_inner();
        let request: MergeLensRequest = match proto_request.try_into() {
            Ok(request) => request,
            Err(e) => return Err(tonic::Status::invalid_argument(e.to_string())),
        };
        let response = LensManagerApi::merge_lens(self, request.into()).await
            .map_err(|e| e.into())?;

        Ok(Response::new(response.into()))
    }
    /// CloseLens will remove a Lens node from the graph, detaching it from its scope
    async fn close_lens(
        &self,
        raw_request: Request<CloseLensRequestProto>,
    ) -> Result<Response<CloseLensResponseProto>, tonic::Status> {
        let proto_request = raw_request.into_inner();
        let request: CloseLensRequest = match proto_request.try_into() {
            Ok(request) => request,
            Err(e) => return Err(tonic::Status::invalid_argument(e.to_string())),
        };
        let response = LensManagerApi::close_lens(self, request.into()).await
            .map_err(|e| e.into())?;

        Ok(Response::new(response.into()))
    }
    /// Adds a given entity node to the scope of a lens
    async fn add_node_to_scope(
        &self,
        raw_request: Request<AddNodeToScopeRequestProto>,
    ) -> Result<Response<AddNodeToScopeResponseProto>, tonic::Status> {
        let proto_request = raw_request.into_inner();
        let request: AddNodeToScopeRequest = match proto_request.try_into() {
            Ok(request) => request,
            Err(e) => return Err(tonic::Status::invalid_argument(e.to_string())),
        };
        let response = LensManagerApi::add_node_to_scope(self, request.into()).await
            .map_err(|e| e.into())?;

        Ok(Response::new(response.into()))
    }
    /// Remove a node from a given lens's scope
    async fn remove_node_from_scope(
        &self,
        raw_request: Request<RemoveNodeFromScopeRequestProto>,
    ) -> Result<Response<RemoveNodeFromScopeResponseProto>, tonic::Status> {
        let proto_request = raw_request.into_inner();
        let request: RemoveNodeFromScopeRequest = match proto_request.try_into() {
            Ok(request) => request,
            Err(e) => return Err(tonic::Status::invalid_argument(e.to_string())),
        };
        let response = LensManagerApi::remove_node_from_scope(self, request.into()).await
            .map_err(|e| e.into())?;

        Ok(Response::new(response.into()))
    }
    /// Remove a node from all of the lens scopes it is attached to
    async fn remove_node_from_all_scopes(
        &self,
        raw_request: Request<RemoveNodeFromAllScopesRequestProto>,
    ) -> Result<Response<RemoveNodeFromAllScopesResponseProto>, tonic::Status> {
        let proto_request = raw_request.into_inner();
        let request: RemoveNodeFromAllScopesRequest = match proto_request.try_into() {
            Ok(request) => request,
            Err(e) => return Err(tonic::Status::invalid_argument(e.to_string())),
        };
        let response = LensManagerApi::remove_node_from_all_scopes(self, request.into()).await
            .map_err(|e| e.into())?;

        Ok(Response::new(response.into()))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LensManagerServiceServerError {
    #[error("grpc transport error: {0}")]
    GrpcTransportError(#[from] tonic::transport::Error),
    #[error("Bind error: {0}")]
    BindError(std::io::Error),
}

/// A server construct that drives the LensManagerApi implementation.
pub struct LensManagerServiceServer<T, E>
where
    T: LensManagerApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    server: LensManagerServiceServerProto<T>,
    addr: SocketAddr,
    shutdown_rx: Receiver<()>,
}

impl<T, E> LensManagerServiceServer<T, E>
where
    T: LensManagerApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    pub fn builder(
        service: T,
        addr: SocketAddr,
        shutdown_rx: Receiver<()>,
    ) -> LensManagerServiceServerBuilder<T, E> {
        LensManagerServiceServerBuilder::new(service, addr, shutdown_rx)
    }

    pub async fn serve(self) -> Result<(), LensManagerServiceServerError> {
        let (healthcheck_handle, health_service) =
            init_health_service::<LensManagerServiceServerProto<T>, _, _>(
                || async { Ok(HealthcheckStatus::Serving) },
                std::time::Duration::from_millis(500),
            )
            .await;

        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(LensManagerServiceServerError::BindError)?;

        Server::builder()
            .trace_fn(|request| {
                tracing::trace_span!(
                    "LensManager",
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

pub struct LensManagerServiceServerBuilder<T, E>
where
    T: LensManagerApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    server: LensManagerServiceServerProto<T>,
    addr: SocketAddr,
    shutdown_rx: Receiver<()>,
}

impl<T, E> LensManagerServiceServerBuilder<T, E>
where
    T: LensManagerApi<Error = E> + Send + Sync + 'static,
    E: Into<Status> + Send + Sync + 'static,
{
    /// Create a new builder for a LensManagerServiceServer,
    /// taking the required arguments upfront.
    pub fn new(service: T, addr: SocketAddr, shutdown_rx: Receiver<()>) -> Self {
        Self {
            server: LensManagerServiceServerProto::new(service),
            addr,
            shutdown_rx,
        }
    }

    /// Consumes the builder and returns a new `LensManagerServiceServer`.
    /// Note: Panics on invalid build state
    pub fn build(self) -> LensManagerServiceServer<T, E> {
        LensManagerServiceServer {
            server: self.server,
            addr: self.addr,
            shutdown_rx: self.shutdown_rx,
        }
    }
}
