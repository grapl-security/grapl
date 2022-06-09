#[cfg(feature = "lens-manager-server")]
pub mod server {
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
        async fn create_lens(
            &self,
            request: CreateLensRequest,
        ) -> Result<CreateLensResponse, Status>;
        /// MergeLens adds the scope of one lens to another
        async fn merge_lens(&self, request: MergeLensRequest) -> Result<MergeLensResponse, Status>;
        /// CloseLens will remove a Lens node from the graph, detaching it from its scope
        async fn close_lens(&self, request: CloseLensRequest) -> Result<CloseLensResponse, Status>;
        /// Adds a given entity node to the scope of a lens
        async fn add_node_to_scope(
            &self,
            request: AddNodeToScopeRequest,
        ) -> Result<AddNodeToScopeResponse, Status>;
        /// Remove a node from a given lens's scope
        async fn remove_node_from_scope(
            &self,
            request: RemoveNodeFromScopeRequest,
        ) -> Result<RemoveNodeFromScopeResponse, Status>;
        /// Remove a node from all of the lens scopes it is attached to
        async fn remove_node_from_all_scopes(
            &self,
            request: RemoveNodeFromAllScopesRequest,
        ) -> Result<RemoveNodeFromAllScopesResponse, Status>;
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
            let response = LensManagerApi::create_lens(self, request.into()).await?;

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
            let response = LensManagerApi::merge_lens(self, request.into()).await?;

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
            let response = LensManagerApi::close_lens(self, request.into()).await?;

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
            let response = LensManagerApi::add_node_to_scope(self, request.into()).await?;

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
            let response = LensManagerApi::remove_node_from_scope(self, request.into()).await?;

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
            let response =
                LensManagerApi::remove_node_from_all_scopes(self, request.into()).await?;

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
}

#[cfg(feature = "lens-manager-client")]
pub mod client {
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
            lens_manager_service_client::LensManagerServiceClient as LensManagerServiceClientProto,
            AddNodeToScopeRequest as AddNodeToScopeRequestProto,
            CloseLensRequest as CloseLensRequestProto,
            CreateLensRequest as CreateLensRequestProto,
            MergeLensRequest as MergeLensRequestProto,
            RemoveNodeFromAllScopesRequest as RemoveNodeFromAllScopesRequestProto,
            RemoveNodeFromScopeRequest as RemoveNodeFromScopeRequestProto,
        },
        protocol::status::Status,
        SerDeError,
    };

    #[derive(thiserror::Error, Debug)]
    pub enum LensManagerServiceClientError {
        #[error("Failed to deserialize response {0}")]
        SerDeError(#[from] SerDeError),
        #[error("Status {0}")]
        Status(Status),
        #[error("ConnectError")]
        ConnectError(tonic::transport::Error),
    }

    #[derive(Clone)]
    pub struct LensManagerServiceClient {
        inner: LensManagerServiceClientProto<tonic::transport::Channel>,
    }

    impl LensManagerServiceClient {
        pub async fn connect<T>(endpoint: T) -> Result<Self, LensManagerServiceClientError>
        where
            T: TryInto<tonic::transport::Endpoint>,
            T::Error: std::error::Error + Send + Sync + 'static,
        {
            Ok(LensManagerServiceClient {
                inner: LensManagerServiceClientProto::connect(endpoint)
                    .await
                    .map_err(LensManagerServiceClientError::ConnectError)?,
            })
        }

        /// Creates a new lens with an empty scope
        pub async fn create_lens(
            &mut self,
            request: CreateLensRequest,
        ) -> Result<CreateLensResponse, LensManagerServiceClientError> {
            let raw_request: CreateLensRequestProto = request.into();
            let raw_response = self
                .inner
                .create_lens(raw_request)
                .await
                .map_err(|s| LensManagerServiceClientError::Status(s.into()))?;
            let proto_response = raw_response.into_inner();
            let response = proto_response.try_into()?;
            Ok(response)
        }

        /// MergeLens adds the scope of one lens to another
        pub async fn merge_lens(
            &mut self,
            request: MergeLensRequest,
        ) -> Result<MergeLensResponse, LensManagerServiceClientError> {
            let raw_request: MergeLensRequestProto = request.into();
            let raw_response = self
                .inner
                .merge_lens(raw_request)
                .await
                .map_err(|s| LensManagerServiceClientError::Status(s.into()))?;
            let proto_response = raw_response.into_inner();
            let response = proto_response.try_into()?;
            Ok(response)
        }

        /// CloseLens will remove a Lens node from the graph, detaching it from its scope
        pub async fn close_lens(
            &mut self,
            request: CloseLensRequest,
        ) -> Result<CloseLensResponse, LensManagerServiceClientError> {
            let raw_request: CloseLensRequestProto = request.into();
            let raw_response = self
                .inner
                .close_lens(raw_request)
                .await
                .map_err(|s| LensManagerServiceClientError::Status(s.into()))?;
            let proto_response = raw_response.into_inner();
            let response = proto_response.try_into()?;
            Ok(response)
        }

        /// Adds a given entity node to the scope of a lens
        pub async fn add_node_to_scope(
            &mut self,
            request: AddNodeToScopeRequest,
        ) -> Result<AddNodeToScopeResponse, LensManagerServiceClientError> {
            let raw_request: AddNodeToScopeRequestProto = request.into();
            let raw_response = self
                .inner
                .add_node_to_scope(raw_request)
                .await
                .map_err(|s| LensManagerServiceClientError::Status(s.into()))?;
            let proto_response = raw_response.into_inner();
            let response = proto_response.try_into()?;
            Ok(response)
        }

        /// Remove a node from a given lens's scope
        pub async fn remove_node_from_scope(
            &mut self,
            request: RemoveNodeFromScopeRequest,
        ) -> Result<RemoveNodeFromScopeResponse, LensManagerServiceClientError> {
            let raw_request: RemoveNodeFromScopeRequestProto = request.into();
            let raw_response = self
                .inner
                .remove_node_from_scope(raw_request)
                .await
                .map_err(|s| LensManagerServiceClientError::Status(s.into()))?;
            let proto_response = raw_response.into_inner();
            let response = proto_response.try_into()?;
            Ok(response)
        }

        /// Remove a node from all of the lens scopes it is attached to
        pub async fn remove_node_from_all_scopes(
            &mut self,
            request: RemoveNodeFromAllScopesRequest,
        ) -> Result<RemoveNodeFromAllScopesResponse, LensManagerServiceClientError> {
            let raw_request: RemoveNodeFromAllScopesRequestProto = request.into();
            let raw_response = self
                .inner
                .remove_node_from_all_scopes(raw_request)
                .await
                .map_err(|s| LensManagerServiceClientError::Status(s.into()))?;
            let proto_response = raw_response.into_inner();
            let response = proto_response.try_into()?;
            Ok(response)
        }
    }
}

#[cfg(feature = "lens-manager-messages")]
pub mod messages {
    use crate::{
        graplinc::common::v1beta1::Uuid,
        protobufs::graplinc::grapl::api::lens_manager::v1beta1::{
            merge_lens_request::MergeBehavior as MergeBehaviorProto,
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
        serde_impl,
        type_url,
        SerDeError,
    };

    //
    // CreateLensRequest
    //

    #[derive(Debug, Clone, PartialEq)]
    pub struct CreateLensRequest {
        pub tenant_id: Uuid,
        pub lens_type: String,
        pub lens_name: String,
        pub is_engagement: bool,
    }

    impl TryFrom<CreateLensRequestProto> for CreateLensRequest {
        type Error = SerDeError;

        fn try_from(request_proto: CreateLensRequestProto) -> Result<Self, Self::Error> {
            let tenant_id = request_proto
                .tenant_id
                .ok_or(SerDeError::MissingField("tenant_id"))?;

            let lens_type = request_proto.lens_type;
            let lens_name = request_proto.lens_name;
            let is_engagement = request_proto.is_engagement;

            Ok(CreateLensRequest {
                tenant_id: tenant_id.into(),
                lens_type,
                lens_name,
                is_engagement,
            })
        }
    }

    impl From<CreateLensRequest> for CreateLensRequestProto {
        fn from(request: CreateLensRequest) -> Self {
            CreateLensRequestProto {
                tenant_id: Some(request.tenant_id.into()),
                lens_type: request.lens_type,
                lens_name: request.lens_name,
                is_engagement: request.is_engagement,
            }
        }
    }

    impl type_url::TypeUrl for CreateLensRequest {
        const TYPE_URL: &'static str =
            "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CreateLensRequest";
    }

    impl serde_impl::ProtobufSerializable for CreateLensRequest {
        type ProtobufMessage = CreateLensRequestProto;
    }

    // //
    // // CreateLensResponse
    // //

    #[derive(Debug, Clone, PartialEq)]
    pub struct CreateLensResponse {
        pub lens_uid: u64,
    }

    impl TryFrom<CreateLensResponseProto> for CreateLensResponse {
        type Error = SerDeError;

        fn try_from(response_proto: CreateLensResponseProto) -> Result<Self, Self::Error> {
            Ok(CreateLensResponse {
                lens_uid: response_proto.lens_uid,
            })
        }
    }

    impl From<CreateLensResponse> for CreateLensResponseProto {
        fn from(response: CreateLensResponse) -> Self {
            CreateLensResponseProto {
                lens_uid: response.lens_uid,
            }
        }
    }

    impl type_url::TypeUrl for CreateLensResponse {
        const TYPE_URL: &'static str =
            "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CreateLensResponse";
    }

    impl serde_impl::ProtobufSerializable for CreateLensResponse {
        type ProtobufMessage = CreateLensResponseProto;
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum MergeBehavior {
        Preserve,
        Close,
    }

    impl TryFrom<MergeBehaviorProto> for MergeBehavior {
        type Error = SerDeError;

        fn try_from(merge_behavior: MergeBehaviorProto) -> Result<Self, Self::Error> {
            match merge_behavior {
                MergeBehaviorProto::Unspecified => Err(SerDeError::UnknownVariant("Unspecified")),
                MergeBehaviorProto::Preserve => Ok(MergeBehavior::Preserve),
                MergeBehaviorProto::Close => Ok(MergeBehavior::Close),
            }
        }
    }

    impl From<MergeBehavior> for MergeBehaviorProto {
        fn from(merge_behavior: MergeBehavior) -> Self {
            match merge_behavior {
                MergeBehavior::Preserve => MergeBehaviorProto::Preserve,
                MergeBehavior::Close => MergeBehaviorProto::Close,
            }
        }
    }

    impl type_url::TypeUrl for MergeBehavior {
        const TYPE_URL: &'static str =
            "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.MergeBehavior";
    }

    //
    // MergeLensRequest
    //

    #[derive(Debug, Clone, PartialEq)]
    pub struct MergeLensRequest {
        pub tenant_id: Uuid,
        pub source_lens_uid: u64,
        pub target_lens_uid: u64,
        pub merge_behavior: MergeBehavior,
    }

    impl TryFrom<MergeLensRequestProto> for MergeLensRequest {
        type Error = SerDeError;

        fn try_from(request_proto: MergeLensRequestProto) -> Result<Self, Self::Error> {
            let merge_behavior = request_proto.merge_behavior().try_into()?;

            let tenant_id = request_proto
                .tenant_id
                .ok_or(SerDeError::MissingField("tenant_id"))?
                .into();

            let source_lens_uid = request_proto.source_lens_uid;

            let target_lens_uid = request_proto.target_lens_uid;

            Ok(MergeLensRequest {
                tenant_id,
                source_lens_uid,
                target_lens_uid,
                merge_behavior,
            })
        }
    }

    impl From<MergeLensRequest> for MergeLensRequestProto {
        fn from(request: MergeLensRequest) -> Self {
            MergeLensRequestProto {
                tenant_id: Some(request.tenant_id.into()),
                source_lens_uid: request.source_lens_uid.into(),
                target_lens_uid: request.target_lens_uid.into(),
                merge_behavior: request.merge_behavior as i32,
            }
        }
    }

    impl type_url::TypeUrl for MergeLensRequest {
        const TYPE_URL: &'static str =
            "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.MergeLensRequest";
    }

    impl serde_impl::ProtobufSerializable for MergeLensRequest {
        type ProtobufMessage = MergeLensRequestProto;
    }

    //
    // MergeLensResponse
    //

    #[derive(Debug, Clone, PartialEq)]
    pub struct MergeLensResponse {}

    impl TryFrom<MergeLensResponseProto> for MergeLensResponse {
        type Error = SerDeError;
        fn try_from(_response_proto: MergeLensResponseProto) -> Result<Self, Self::Error> {
            Ok(Self {})
        }
    }

    impl From<MergeLensResponse> for MergeLensResponseProto {
        fn from(_request: MergeLensResponse) -> Self {
            MergeLensResponseProto {}
        }
    }

    impl type_url::TypeUrl for MergeLensResponse {
        const TYPE_URL: &'static str =
            "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.MergeLensResponse";
    }

    impl serde_impl::ProtobufSerializable for MergeLensResponse {
        type ProtobufMessage = MergeLensResponseProto;
    }

    //
    //  CloseLensRequest
    //

    #[derive(Debug, Clone, PartialEq)]
    pub struct CloseLensRequest {
        pub tenant_id: Uuid,
        pub lens_uid: u64,
    }

    impl TryFrom<CloseLensRequestProto> for CloseLensRequest {
        type Error = SerDeError;

        fn try_from(request_proto: CloseLensRequestProto) -> Result<Self, Self::Error> {
            let tenant_id = request_proto
                .tenant_id
                .ok_or(SerDeError::MissingField("tenant_id"))?
                .into();

            let lens_uid = request_proto.lens_uid;

            Ok(CloseLensRequest {
                tenant_id,
                lens_uid,
            })
        }
    }

    impl From<CloseLensRequest> for CloseLensRequestProto {
        fn from(request: CloseLensRequest) -> Self {
            CloseLensRequestProto {
                tenant_id: Some(request.tenant_id.into()),
                lens_uid: request.lens_uid,
            }
        }
    }

    impl type_url::TypeUrl for CloseLensRequest {
        const TYPE_URL: &'static str =
            "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CloseLensRequest";
    }

    impl serde_impl::ProtobufSerializable for CloseLensRequest {
        type ProtobufMessage = CloseLensRequestProto;
    }

    //
    // CloseLensResponse
    //

    #[derive(Debug, Clone, PartialEq)]
    pub struct CloseLensResponse {}

    impl TryFrom<CloseLensResponseProto> for CloseLensResponse {
        type Error = SerDeError;

        fn try_from(_response_proto: CloseLensResponseProto) -> Result<Self, Self::Error> {
            Ok(Self {})
        }
    }

    impl From<CloseLensResponse> for CloseLensResponseProto {
        fn from(_request: CloseLensResponse) -> Self {
            CloseLensResponseProto {}
        }
    }

    impl type_url::TypeUrl for CloseLensResponse {
        const TYPE_URL: &'static str =
            "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CloseLensResponse";
    }

    impl serde_impl::ProtobufSerializable for CloseLensResponse {
        type ProtobufMessage = CloseLensResponseProto;
    }

    //
    // AddNodeToScopeRequest
    //

    #[derive(Debug, Clone, PartialEq)]
    pub struct AddNodeToScopeRequest {
        pub tenant_id: Uuid,
        pub lens_uid: u64,
        pub uid: u64,
    }

    impl TryFrom<AddNodeToScopeRequestProto> for AddNodeToScopeRequest {
        type Error = SerDeError;

        fn try_from(request_proto: AddNodeToScopeRequestProto) -> Result<Self, Self::Error> {
            let tenant_id = request_proto
                .tenant_id
                .ok_or(SerDeError::MissingField("tenant_id"))?;

            let lens_uid = request_proto.lens_uid;

            let uid = request_proto.uid;

            Ok(AddNodeToScopeRequest {
                tenant_id: tenant_id.into(),
                lens_uid,
                uid,
            })
        }
    }

    impl From<AddNodeToScopeRequest> for AddNodeToScopeRequestProto {
        fn from(request: AddNodeToScopeRequest) -> Self {
            AddNodeToScopeRequestProto {
                tenant_id: Some(request.tenant_id.into()),
                lens_uid: request.lens_uid,
                uid: request.uid.into(),
            }
        }
    }

    impl type_url::TypeUrl for AddNodeToScopeRequest {
        const TYPE_URL: &'static str =
            "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.AddNodeToScopeRequest";
    }

    impl serde_impl::ProtobufSerializable for AddNodeToScopeRequest {
        type ProtobufMessage = AddNodeToScopeRequestProto;
    }

    // AddNodeToScopeResponse
    //

    #[derive(Debug, Clone, PartialEq)]
    pub struct AddNodeToScopeResponse {}

    impl TryFrom<AddNodeToScopeResponseProto> for AddNodeToScopeResponse {
        type Error = SerDeError;
        fn try_from(_response_proto: AddNodeToScopeResponseProto) -> Result<Self, Self::Error> {
            Ok(Self {})
        }
    }

    impl From<AddNodeToScopeResponse> for AddNodeToScopeResponseProto {
        fn from(_response: AddNodeToScopeResponse) -> Self {
            AddNodeToScopeResponseProto {}
        }
    }

    impl type_url::TypeUrl for AddNodeToScopeResponse {
        const TYPE_URL: &'static str =
            "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.AddNodeToScopeResponse";
    }

    impl serde_impl::ProtobufSerializable for AddNodeToScopeResponse {
        type ProtobufMessage = AddNodeToScopeResponseProto;
    }

    //
    // RemoveNodeFromScopeRequest
    //
    #[derive(Debug, Clone, PartialEq)]
    pub struct RemoveNodeFromScopeRequest {
        pub tenant_id: Uuid,
        pub lens_uid: u64,
        pub uid: u64,
    }

    impl TryFrom<RemoveNodeFromScopeRequestProto> for RemoveNodeFromScopeRequest {
        type Error = SerDeError;

        fn try_from(request_proto: RemoveNodeFromScopeRequestProto) -> Result<Self, Self::Error> {
            let tenant_id = request_proto
                .tenant_id
                .ok_or(SerDeError::MissingField("tenant_id"))?;

            let lens_uid = request_proto.lens_uid;

            Ok(RemoveNodeFromScopeRequest {
                tenant_id: tenant_id.into(),
                lens_uid,
                uid: request_proto.uid,
            })
        }
    }

    impl From<RemoveNodeFromScopeRequest> for RemoveNodeFromScopeRequestProto {
        fn from(request: RemoveNodeFromScopeRequest) -> Self {
            RemoveNodeFromScopeRequestProto {
                tenant_id: Some(request.tenant_id.into()),
                lens_uid: request.lens_uid,
                uid: request.uid,
            }
        }
    }

    impl type_url::TypeUrl for RemoveNodeFromScopeRequest {
        const TYPE_URL: &'static str =
            "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.RemoveNodeFromScopeRequest";
    }

    impl serde_impl::ProtobufSerializable for RemoveNodeFromScopeRequest {
        type ProtobufMessage = RemoveNodeFromScopeRequestProto;
    }

    //
    // RemoveNodeFromScopeResponse
    //

    #[derive(Debug, Clone, PartialEq)]
    pub struct RemoveNodeFromScopeResponse {}

    impl TryFrom<RemoveNodeFromScopeResponseProto> for RemoveNodeFromScopeResponse {
        type Error = SerDeError;
        fn try_from(
            _response_proto: RemoveNodeFromScopeResponseProto,
        ) -> Result<Self, Self::Error> {
            Ok(Self {})
        }
    }

    impl From<RemoveNodeFromScopeResponse> for RemoveNodeFromScopeResponseProto {
        fn from(_request: RemoveNodeFromScopeResponse) -> Self {
            RemoveNodeFromScopeResponseProto {}
        }
    }

    impl type_url::TypeUrl for RemoveNodeFromScopeResponse {
        const TYPE_URL: &'static str =
            "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.RemoveNodeFromScopeResponse";
    }

    impl serde_impl::ProtobufSerializable for RemoveNodeFromScopeResponse {
        type ProtobufMessage = RemoveNodeFromScopeResponseProto;
    }

    //
    // RemoveNodeFromAllScopesRequest
    //

    #[derive(Debug, Clone, PartialEq)]
    pub struct RemoveNodeFromAllScopesRequest {
        pub tenant_id: Uuid,
        //
        pub uid: u64, // never 0, Refinement Type
    }

    impl TryFrom<RemoveNodeFromAllScopesRequestProto> for RemoveNodeFromAllScopesRequest {
        type Error = SerDeError;

        fn try_from(
            request_proto: RemoveNodeFromAllScopesRequestProto,
        ) -> Result<Self, Self::Error> {
            let tenant_id = request_proto
                .tenant_id
                .ok_or(SerDeError::MissingField("tenant_id"))?;

            let uid = request_proto.uid;

            // check for invalid values for u64 and send back error otherwise deserailize
            Ok(RemoveNodeFromAllScopesRequest {
                tenant_id: tenant_id.into(),
                uid,
            })
        }
    }

    impl From<RemoveNodeFromAllScopesRequest> for RemoveNodeFromAllScopesRequestProto {
        fn from(request: RemoveNodeFromAllScopesRequest) -> Self {
            RemoveNodeFromAllScopesRequestProto {
                tenant_id: Some(request.tenant_id.into()),
                uid: request.uid.into(),
            }
        }
    }

    impl type_url::TypeUrl for RemoveNodeFromAllScopesRequest {
        const TYPE_URL: &'static str =
            "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.RemoveNodeFromAllScopesRequest";
    }

    impl serde_impl::ProtobufSerializable for RemoveNodeFromAllScopesRequest {
        type ProtobufMessage = RemoveNodeFromAllScopesRequestProto;
    }

    //
    // RemoveNodeFromAllScopesResponse
    //

    #[derive(Debug, Clone, PartialEq)]
    pub struct RemoveNodeFromAllScopesResponse {}

    impl TryFrom<RemoveNodeFromAllScopesResponseProto> for RemoveNodeFromAllScopesResponse {
        type Error = SerDeError;

        fn try_from(_response: RemoveNodeFromAllScopesResponseProto) -> Result<Self, Self::Error> {
            Ok(Self {})
        }
    }

    impl From<RemoveNodeFromAllScopesResponse> for RemoveNodeFromAllScopesResponseProto {
        fn from(_response: RemoveNodeFromAllScopesResponse) -> Self {
            RemoveNodeFromAllScopesResponseProto {}
        }
    }

    impl type_url::TypeUrl for RemoveNodeFromAllScopesResponse {
        const TYPE_URL: &'static str =
            "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.RemoveNodeFromAllScopesResponse";
    }

    impl serde_impl::ProtobufSerializable for RemoveNodeFromAllScopesResponse {
        type ProtobufMessage = RemoveNodeFromAllScopesResponseProto;
    }
}
