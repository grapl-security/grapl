// todo: When rust-proto-new supports its own Status type we can replace the use of Status in this module

#[cfg(feature = "uid-allocator-server")]
pub mod server {
    use std::net::SocketAddr;

    use tonic::{
        transport::Server,
        Request,
        Response,
        Status,
    };

    use crate::{
        graplinc::grapl::api::uid_allocator::v1beta1::messages::{
            AllocateIdsRequest,
            AllocateIdsResponse,
        },
        protobufs::graplinc::grapl::api::uid_allocator::v1beta1::{
            uid_allocator_server::{
                UidAllocator as UidAllocatorProto,
                UidAllocatorServer as UidAllocatorServerProto,
            },
            AllocateIdsRequest as AllocateIdsRequestProto,
            AllocateIdsResponse as AllocateIdsResponseProto,
        },
    };

    #[async_trait::async_trait]
    pub trait UidAllocatorApi {
        type Error: Into<Status>;
        // todo: swap out for rust-proto-new::Status when it's available
        /// Requests a new allocation of Uids for a given tenant
        /// Note that it may not always return the requested size, but it will
        /// never return an empty allocation
        async fn allocate_ids(
            &self,
            request: AllocateIdsRequest,
        ) -> Result<AllocateIdsResponse, Self::Error>;
    }

    #[async_trait::async_trait]
    impl<T, E> UidAllocatorProto for T
    where
        T: UidAllocatorApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
    {
        async fn allocate_ids(
            &self,
            raw_request: Request<AllocateIdsRequestProto>,
        ) -> Result<Response<AllocateIdsResponseProto>, Status> {
            let proto_request = raw_request.into_inner();
            let request: AllocateIdsRequest = match proto_request.try_into() {
                Ok(request) => request,
                Err(e) => return Err(Status::invalid_argument(e.to_string())),
            };
            let response = UidAllocatorApi::allocate_ids(self, request)
                .await
                .map_err(|e| e.into())?;

            Ok(Response::new(response.into()))
        }
    }

    #[derive(thiserror::Error, Debug)]
    pub enum UidAllocatorServerError {
        #[error("grpc transport error: {0}")]
        GrpcTransportError(#[from] tonic::transport::Error),
    }

    /// A server construct that drives the UidAllocatorApi implementation.
    pub struct UidAllocatorServer<T, E>
    where
        T: UidAllocatorApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
    {
        server: UidAllocatorServerProto<T>,
        addr: SocketAddr,
    }

    impl<T, E> UidAllocatorServer<T, E>
    where
        T: UidAllocatorApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
    {
        pub fn builder(service: T, addr: SocketAddr) -> UidAllocatorServerBuilder<T, E> {
            UidAllocatorServerBuilder::new(service, addr)
        }

        pub async fn serve(&mut self) -> Result<(), UidAllocatorServerError> {
            Server::builder()
                // todo: healthchecks and whatnot
                .trace_fn(|request| {
                    tracing::trace_span!(
                        "UidAllocator",
                        headers = ?request.headers(),
                        method = ?request.method(),
                        uri = %request.uri(),
                        extensions = ?request.extensions(),
                    )
                })
                .add_service(self.server.clone())
                .serve(self.addr)
                .await?;
            Ok(())
        }
    }

    pub struct UidAllocatorServerBuilder<T, E>
    where
        T: UidAllocatorApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
    {
        server: UidAllocatorServerProto<T>,
        addr: SocketAddr,
    }

    impl<T, E> UidAllocatorServerBuilder<T, E>
    where
        T: UidAllocatorApi<Error = E> + Send + Sync + 'static,
        E: Into<Status> + Send + Sync + 'static,
    {
        /// Create a new builder for a UidAllocatorServer,
        /// taking the required arguments upfront.
        pub fn new(service: T, addr: SocketAddr) -> Self {
            Self {
                server: UidAllocatorServerProto::new(service),
                addr,
            }
        }

        /// Consumes the builder and returns a new `UidAllocatorServer`.
        /// Note: Panics on invalid build state
        pub fn build(self) -> UidAllocatorServer<T, E> {
            UidAllocatorServer {
                server: self.server,
                addr: self.addr,
            }
        }
    }
}

#[cfg(feature = "uid-allocator-client")]
pub mod client {
    use tonic::Status;

    use crate::{
        graplinc::grapl::api::uid_allocator::v1beta1::messages::{
            AllocateIdsRequest,
            AllocateIdsResponse,
        },
        protobufs::graplinc::grapl::api::uid_allocator::v1beta1::{
            uid_allocator_client::UidAllocatorClient as UidAllocatorClientProto,
            AllocateIdsRequest as AllocateIdsRequestProto,
        },
        SerDeError,
    };

    #[derive(thiserror::Error, Debug)]
    pub enum UidAllocatorClientError {
        #[error("Failed to deserialize response {0}")]
        SerDeError(#[from] SerDeError),
        #[error("GrpcError {0}")]
        GrpcError(#[from] Status),
        #[error("ConnectError")]
        ConnectError(tonic::transport::Error),
    }

    #[derive(Clone)]
    pub struct UidAllocatorClient {
        inner: UidAllocatorClientProto<tonic::transport::Channel>,
    }

    impl UidAllocatorClient {
        pub async fn connect<T>(endpoint: T) -> Result<Self, UidAllocatorClientError>
        where
            T: std::convert::TryInto<tonic::transport::Endpoint>,
            T::Error: std::error::Error + Send + Sync + 'static,
        {
            Ok(UidAllocatorClient {
                inner: UidAllocatorClientProto::connect(endpoint)
                    .await
                    .map_err(UidAllocatorClientError::ConnectError)?,
            })
        }

        pub async fn allocate_ids(
            &mut self,
            request: AllocateIdsRequest,
        ) -> Result<AllocateIdsResponse, UidAllocatorClientError> {
            let raw_request: AllocateIdsRequestProto = request.into();
            let raw_response = self.inner.allocate_ids(raw_request).await?;
            let proto_response = raw_response.into_inner();
            let response = proto_response.try_into()?;
            Ok(response)
        }
    }
}

#[cfg(feature = "uid-allocator-messages")]
pub mod messages {
    use crate::{
        protobufs::graplinc::grapl::api::uid_allocator::v1beta1::{
            AllocateIdsRequest as AllocateIdsRequestProto,
            AllocateIdsResponse as AllocateIdsResponseProto,
            Allocation as AllocationProto,
        },
        type_url,
        SerDeError,
    };

    #[derive(Copy, Clone, Debug)]
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

    #[derive(Clone, Debug)]
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

    #[derive(Clone, Debug)]
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
