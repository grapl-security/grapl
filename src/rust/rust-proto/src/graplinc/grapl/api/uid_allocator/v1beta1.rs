pub mod server {
    use std::{
        future::Future,
        time::Duration,
    };

    use futures::FutureExt;
    use tokio::{
        net::TcpListener,
        sync::oneshot::Receiver,
    };
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
            uid_allocator::v1beta1::messages::{
                AllocateIdsRequest,
                AllocateIdsResponse,
                CreateTenantKeyspaceRequest,
                CreateTenantKeyspaceResponse,
            },
        },
        protobufs::graplinc::grapl::api::uid_allocator::v1beta1::{
            uid_allocator_service_server::{
                UidAllocatorService as UidAllocatorServiceProto,
                UidAllocatorServiceServer,
            },
            AllocateIdsRequest as AllocateIdsRequestProto,
            AllocateIdsResponse as AllocateIdsResponseProto,
            CreateTenantKeyspaceRequest as CreateTenantKeyspaceRequestProto,
            CreateTenantKeyspaceResponse as CreateTenantKeyspaceResponseProto,
        },
    };

    #[async_trait::async_trait]
    pub trait UidAllocatorApi {
        type Error: Into<Status>;

        /// Requests a new allocation of Uids for a given tenant
        /// Note that it may not always return the requested size, but it will
        /// never return an empty allocation
        async fn allocate_ids(
            &self,
            request: AllocateIdsRequest,
        ) -> Result<AllocateIdsResponse, Self::Error>;

        async fn create_tenant_keyspace(
            &self,
            request: CreateTenantKeyspaceRequest,
        ) -> Result<CreateTenantKeyspaceResponse, Self::Error>;
    }

    #[async_trait::async_trait]
    impl<T> UidAllocatorServiceProto for GrpcApi<T>
    where
        T: UidAllocatorApi + Send + Sync + 'static,
    {
        async fn allocate_ids(
            &self,
            request: Request<AllocateIdsRequestProto>,
        ) -> Result<Response<AllocateIdsResponseProto>, tonic::Status> {
            execute_rpc!(self, request, allocate_ids)
        }

        async fn create_tenant_keyspace(
            &self,
            request: Request<CreateTenantKeyspaceRequestProto>,
        ) -> Result<Response<CreateTenantKeyspaceResponseProto>, tonic::Status> {
            execute_rpc!(self, request, create_tenant_keyspace)
        }
    }

    /**
     * !!!!! IMPORTANT !!!!!
     * This is almost entirely cargo-culted from PipelineIngressServer.
     * Lots of opportunities to deduplicate and simplify.
     */
    pub struct UidAllocatorServer<T, H, F>
    where
        T: UidAllocatorApi + Send + Sync + 'static,
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

    impl<T, H, F> UidAllocatorServer<T, H, F>
    where
        T: UidAllocatorApi + Send + Sync + 'static,
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
        ) -> (Self, tokio::sync::oneshot::Sender<()>) {
            let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
            (
                Self {
                    api_server,
                    healthcheck,
                    healthcheck_polling_interval,
                    tcp_listener,
                    shutdown_rx,
                    service_name: UidAllocatorServiceServer::<GrpcApi<T>>::NAME,
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
                init_health_service::<UidAllocatorServiceServer<GrpcApi<T>>, _, _>(
                    self.healthcheck,
                    self.healthcheck_polling_interval,
                )
                .await;

            // TODO: add tower tracing, tls_config, concurrency limits
            Ok(Server::builder()
                .trace_fn(|request| {
                    tracing::info_span!(
                        "UidAllocator",
                        request_id = ?request.headers().get("x-request-id"),
                        method = ?request.method(),
                        uri = %request.uri(),
                        extensions = ?request.extensions(),
                    )
                })
                .add_service(health_service)
                .add_service(UidAllocatorServiceServer::new(GrpcApi::new(
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
}

pub mod client {
    use tonic::transport::Endpoint;

    use crate::{
        graplinc::grapl::api::{
            client::{
                Client,
                ClientError,
                Connectable,
                WithClient,
            },
            uid_allocator::v1beta1::messages as native,
        },
        protobufs::graplinc::grapl::api::uid_allocator::v1beta1::uid_allocator_service_client::UidAllocatorServiceClient,
    };

    #[async_trait::async_trait]
    impl Connectable for UidAllocatorServiceClient<tonic::transport::Channel> {
        async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
            Ok(Self::connect(endpoint).await?)
        }
    }

    #[derive(Clone)]
    /// This allocator should rarely be used. Instead, use the CachingUidAllocatorServiceClient
    /// from the uid-allocator crate.
    pub struct UidAllocatorClient {
        client: Client<UidAllocatorServiceClient<tonic::transport::Channel>>,
    }

    impl WithClient<UidAllocatorServiceClient<tonic::transport::Channel>> for UidAllocatorClient {
        fn with_client(
            client: Client<UidAllocatorServiceClient<tonic::transport::Channel>>,
        ) -> Self {
            Self { client }
        }
    }

    impl UidAllocatorClient {
        pub async fn allocate_ids(
            &mut self,
            request: native::AllocateIdsRequest,
        ) -> Result<native::AllocateIdsResponse, ClientError> {
            self.client
                .execute(
                    request,
                    None,
                    |status| status.code() == tonic::Code::Unavailable,
                    10,
                    |mut client, request| async move { client.allocate_ids(request).await },
                )
                .await
        }

        pub async fn create_tenant_keyspace(
            &mut self,
            request: native::CreateTenantKeyspaceRequest,
        ) -> Result<native::CreateTenantKeyspaceResponse, ClientError> {
            self
                .client
                .execute(
                    request,
                    None,
                    |status| status.code() == tonic::Code::Unavailable,
                    10,
                    |mut client, request| async move { client.create_tenant_keyspace(request).await },
                )
                .await
        }
    }
}

pub mod messages {
    use crate::{
        protobufs::graplinc::grapl::api::uid_allocator::v1beta1::{
            AllocateIdsRequest as AllocateIdsRequestProto,
            AllocateIdsResponse as AllocateIdsResponseProto,
            Allocation as AllocationProto,
            CreateTenantKeyspaceRequest as CreateTenantKeyspaceRequestProto,
            CreateTenantKeyspaceResponse as CreateTenantKeyspaceResponseProto,
        },
        serde_impl,
        type_url,
        SerDeError,
    };

    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct CreateTenantKeyspaceRequest {
        pub tenant_id: uuid::Uuid,
    }

    impl TryFrom<CreateTenantKeyspaceRequestProto> for CreateTenantKeyspaceRequest {
        type Error = SerDeError;
        fn try_from(value: CreateTenantKeyspaceRequestProto) -> Result<Self, Self::Error> {
            Ok(Self {
                tenant_id: value
                    .tenant_id
                    .ok_or_else(|| SerDeError::MissingField("value.tenant_id"))?
                    .into(),
            })
        }
    }

    impl From<CreateTenantKeyspaceRequest> for CreateTenantKeyspaceRequestProto {
        fn from(value: CreateTenantKeyspaceRequest) -> Self {
            Self {
                tenant_id: Some(value.tenant_id.into()),
            }
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct CreateTenantKeyspaceResponse {}

    impl From<CreateTenantKeyspaceResponseProto> for CreateTenantKeyspaceResponse {
        fn from(_value: CreateTenantKeyspaceResponseProto) -> Self {
            Self {}
        }
    }

    impl From<CreateTenantKeyspaceResponse> for CreateTenantKeyspaceResponseProto {
        fn from(_value: CreateTenantKeyspaceResponse) -> Self {
            Self {}
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Allocation {
        pub start: u64,
        pub offset: u32,
    }

    impl TryFrom<AllocationProto> for Allocation {
        type Error = SerDeError;

        fn try_from(proto: AllocationProto) -> Result<Self, Self::Error> {
            Ok(Self {
                start: proto.start,
                offset: proto.offset,
            })
        }
    }

    impl From<Allocation> for AllocationProto {
        fn from(allocation: Allocation) -> Self {
            Self {
                start: allocation.start,
                offset: allocation.offset,
            }
        }
    }

    impl Iterator for Allocation {
        type Item = u64;

        fn next(&mut self) -> Option<Self::Item> {
            if self.start == (self.offset as u64) {
                None
            } else {
                let result = self.start;
                self.start += 1;
                Some(result)
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct AllocateIdsRequest {
        pub count: u32,
        pub tenant_id: uuid::Uuid,
    }

    impl TryFrom<AllocateIdsRequestProto> for AllocateIdsRequest {
        type Error = SerDeError;

        fn try_from(proto: AllocateIdsRequestProto) -> Result<Self, Self::Error> {
            let tenant_id = proto
                .tenant_id
                .ok_or_else(|| SerDeError::MissingField("tenant_id"))?
                .into();

            Ok(Self {
                count: proto.count,
                tenant_id,
            })
        }
    }

    impl From<AllocateIdsRequest> for AllocateIdsRequestProto {
        fn from(request: AllocateIdsRequest) -> Self {
            Self {
                count: request.count,
                tenant_id: Some(request.tenant_id.into()),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct AllocateIdsResponse {
        pub allocation: Allocation,
    }

    impl TryFrom<AllocateIdsResponseProto> for AllocateIdsResponse {
        type Error = SerDeError;

        fn try_from(proto: AllocateIdsResponseProto) -> Result<Self, Self::Error> {
            let allocation = proto
                .allocation
                .ok_or_else(|| SerDeError::MissingField("allocation"))?
                .try_into()?;

            Ok(Self { allocation })
        }
    }

    impl From<AllocateIdsResponse> for AllocateIdsResponseProto {
        fn from(response: AllocateIdsResponse) -> Self {
            Self {
                allocation: Some(response.allocation.into()),
            }
        }
    }

    impl serde_impl::ProtobufSerializable for CreateTenantKeyspaceRequest {
        type ProtobufMessage = CreateTenantKeyspaceRequestProto;
    }

    impl serde_impl::ProtobufSerializable for CreateTenantKeyspaceResponse {
        type ProtobufMessage = CreateTenantKeyspaceResponseProto;
    }

    impl serde_impl::ProtobufSerializable for Allocation {
        type ProtobufMessage = AllocationProto;
    }

    impl serde_impl::ProtobufSerializable for AllocateIdsRequest {
        type ProtobufMessage = AllocateIdsRequestProto;
    }

    impl serde_impl::ProtobufSerializable for AllocateIdsResponse {
        type ProtobufMessage = AllocateIdsResponseProto;
    }

    impl type_url::TypeUrl for CreateTenantKeyspaceRequest {
        const TYPE_URL: &'static str =
            "graplinc.grapl.api.uid_allocator.v1beta1.CreateTenantKeyspaceRequest";
    }

    impl type_url::TypeUrl for CreateTenantKeyspaceResponse {
        const TYPE_URL: &'static str =
            "graplinc.grapl.api.uid_allocator.v1beta1.CreateTenantKeyspaceResponse";
    }

    impl type_url::TypeUrl for Allocation {
        const TYPE_URL: &'static str = "graplinc.grapl.api.uid_allocator.v1beta1.Allocation";
    }

    impl type_url::TypeUrl for AllocateIdsRequest {
        const TYPE_URL: &'static str =
            "graplinc.grapl.api.uid_allocator.v1beta1.AllocateIdsRequest";
    }

    impl type_url::TypeUrl for AllocateIdsResponse {
        const TYPE_URL: &'static str =
            "graplinc.grapl.api.uid_allocator.v1beta1.AllocateIdsResponse";
    }
}
