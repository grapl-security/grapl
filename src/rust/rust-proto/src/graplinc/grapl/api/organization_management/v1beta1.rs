use bytes::Bytes;
use uuid::Uuid;

use crate::{
    protobufs::graplinc::grapl::api::organization_management::v1beta1::{
        CreateOrganizationRequest as CreateOrganizationRequestProto,
        CreateOrganizationResponse as CreateOrganizationResponseProto,
        CreateUserRequest as CreateUserRequestProto,
        CreateUserResponse as CreateUserResponseProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

//
// CreateOrganizationRequest
//

#[derive(Debug, PartialEq, Clone)]
pub struct CreateOrganizationRequest {
    pub organization_display_name: String,
    pub admin_username: String,
    pub admin_email: String,
    pub admin_password: Bytes,
    pub should_reset_password: bool,
}

impl From<CreateOrganizationRequestProto> for CreateOrganizationRequest {
    fn from(create_organization_request_proto: CreateOrganizationRequestProto) -> Self {
        CreateOrganizationRequest {
            organization_display_name: create_organization_request_proto.organization_display_name,
            admin_username: create_organization_request_proto.admin_username,
            admin_email: create_organization_request_proto.admin_email,
            admin_password: create_organization_request_proto.admin_password.into(),
            should_reset_password: create_organization_request_proto.should_reset_password,
        }
    }
}

impl From<CreateOrganizationRequest> for CreateOrganizationRequestProto {
    fn from(create_organization_request: CreateOrganizationRequest) -> Self {
        CreateOrganizationRequestProto {
            organization_display_name: create_organization_request.organization_display_name,
            admin_username: create_organization_request.admin_username,
            admin_email: create_organization_request.admin_email,
            admin_password: create_organization_request.admin_password,
            should_reset_password: create_organization_request.should_reset_password,
        }
    }
}

impl type_url::TypeUrl for CreateOrganizationRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.organization_management.v1beta1.CreateOrganizationRequest";
}

impl serde_impl::ProtobufSerializable for CreateOrganizationRequest {
    type ProtobufMessage = CreateOrganizationRequestProto;
}

//
// CreateOrganizationResponse
//

#[derive(Debug, PartialEq, Clone)]
pub struct CreateOrganizationResponse {
    pub organization_id: Uuid,
}

impl TryFrom<CreateOrganizationResponseProto> for CreateOrganizationResponse {
    type Error = SerDeError;

    fn try_from(
        create_organization_response_proto: CreateOrganizationResponseProto,
    ) -> Result<Self, Self::Error> {
        match create_organization_response_proto.organization_id {
            Some(organization_id) => Ok(CreateOrganizationResponse {
                organization_id: organization_id.into(),
            }),
            None => Err(SerDeError::MissingField("organization_id")),
        }
    }
}

impl From<CreateOrganizationResponse> for CreateOrganizationResponseProto {
    fn from(create_organization_response: CreateOrganizationResponse) -> Self {
        CreateOrganizationResponseProto {
            organization_id: Some(create_organization_response.organization_id.into()),
        }
    }
}

impl type_url::TypeUrl for CreateOrganizationResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.organization_management.v1beta1.CreateOrganizationResponse";
}

impl serde_impl::ProtobufSerializable for CreateOrganizationResponse {
    type ProtobufMessage = CreateOrganizationResponseProto;
}

//
// CreateUserRequest
//

#[derive(Debug, PartialEq, Clone)]
pub struct CreateUserRequest {
    pub organization_id: Uuid,
    pub name: String,
    pub email: String,
    pub password: Bytes,
}

impl TryFrom<CreateUserRequestProto> for CreateUserRequest {
    type Error = SerDeError;

    fn try_from(create_user_request_proto: CreateUserRequestProto) -> Result<Self, Self::Error> {
        match create_user_request_proto.organization_id {
            Some(organization_id) => Ok(CreateUserRequest {
                organization_id: organization_id.into(),
                name: create_user_request_proto.name,
                email: create_user_request_proto.email,
                password: create_user_request_proto.password,
            }),
            None => Err(SerDeError::MissingField("organization_id")),
        }
    }
}

impl From<CreateUserRequest> for CreateUserRequestProto {
    fn from(create_user_request: CreateUserRequest) -> Self {
        CreateUserRequestProto {
            organization_id: Some(create_user_request.organization_id.into()),
            name: create_user_request.name,
            email: create_user_request.email,
            password: create_user_request.password,
        }
    }
}

impl type_url::TypeUrl for CreateUserRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.organization_management.v1beta1.CreateUserRequest";
}

impl serde_impl::ProtobufSerializable for CreateUserRequest {
    type ProtobufMessage = CreateUserRequestProto;
}

//
// CreateUserResponse
//

#[derive(Debug, PartialEq, Clone)]
pub struct CreateUserResponse {
    pub user_id: Uuid,
}

impl TryFrom<CreateUserResponseProto> for CreateUserResponse {
    type Error = SerDeError;

    fn try_from(create_user_response_proto: CreateUserResponseProto) -> Result<Self, Self::Error> {
        match create_user_response_proto.user_id {
            Some(user_id) => Ok(CreateUserResponse {
                user_id: user_id.into(),
            }),
            None => Err(SerDeError::MissingField("user_id")),
        }
    }
}

impl From<CreateUserResponse> for CreateUserResponseProto {
    fn from(create_user_response: CreateUserResponse) -> Self {
        CreateUserResponseProto {
            user_id: Some(create_user_response.user_id.into()),
        }
    }
}

impl type_url::TypeUrl for CreateUserResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.organization_management.v1beta1.CreateUserResponse";
}

impl serde_impl::ProtobufSerializable for CreateUserResponse {
    type ProtobufMessage = CreateUserResponseProto;
}

//
// client
//

pub mod client {
    use std::time::Duration;

    use client_executor::{
        Executor,
        ExecutorConfig,
    };

    use crate::{
        create_proto_client,
        execute_client_rpc,
        graplinc::grapl::api::organization_management::v1beta1 as native,
        protobufs::graplinc::grapl::api::organization_management::{
            v1beta1 as proto,
            v1beta1::organization_management_service_client::OrganizationManagementServiceClient as OrganizationManagementServiceClientProto,
        },
        protocol::{
            endpoint::Endpoint,
            error::GrpcClientError,
            service_client::{
                ConnectError,
                Connectable,
            },
        },
    };

    pub type OrganizationManagementClientError = GrpcClientError;

    pub struct OrganizationManagementClient {
        executor: Executor,
        proto_client: OrganizationManagementServiceClientProto<tonic::transport::Channel>,
    }

    #[async_trait::async_trait]
    impl Connectable for OrganizationManagementClient {
        const SERVICE_NAME: &'static str =
            "graplinc.grapl.api.organization_management.v1beta1.OrganizationManagementService";

        async fn connect(endpoint: Endpoint) -> Result<Self, ConnectError> {
            let executor = Executor::new(ExecutorConfig::new(Duration::from_secs(30)));
            let proto_client = create_proto_client!(
                executor,
                OrganizationManagementServiceClientProto<tonic::transport::Channel>,
                endpoint,
            );

            Ok(Self {
                executor,
                proto_client,
            })
        }
    }

    impl OrganizationManagementClient {
        pub async fn create_organization(
            &mut self,
            request: native::CreateOrganizationRequest,
        ) -> Result<native::CreateOrganizationResponse, OrganizationManagementClientError> {
            execute_client_rpc!(
                self,
                request,
                create_organization,
                proto::CreateOrganizationRequest,
                native::CreateOrganizationResponse,
            )
        }

        pub async fn create_user(
            &mut self,
            request: native::CreateUserRequest,
        ) -> Result<native::CreateUserResponse, OrganizationManagementClientError> {
            execute_client_rpc!(
                self,
                request,
                create_user,
                proto::CreateUserRequest,
                native::CreateUserResponse,
            )
        }
    }
}

//
// server
//

pub mod server {
    use std::time::Duration;

    use futures::{
        channel::oneshot::{
            self,
            Receiver,
        },
        Future,
        FutureExt,
    };
    use tokio::net::TcpListener;
    use tokio_stream::wrappers::TcpListenerStream;
    use tonic::transport::{
        NamedService,
        Server,
    };

    use super::{
        CreateOrganizationRequest,
        CreateOrganizationResponse,
        CreateUserRequest,
        CreateUserResponse,
    };
    use crate::{
        protobufs::graplinc::grapl::api::organization_management::v1beta1::{
            organization_management_service_server::{
                OrganizationManagementService as OrganizationManagementServiceProto,
                OrganizationManagementServiceServer as OrganizationManagementServiceServerProto,
            },
            CreateOrganizationRequest as CreateOrganizationRequestProto,
            CreateOrganizationResponse as CreateOrganizationResponseProto,
            CreateUserRequest as CreateUserRequestProto,
            CreateUserResponse as CreateUserResponseProto,
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

    //
    // protocol buffer translation layer
    //

    #[tonic::async_trait]
    impl<T> OrganizationManagementServiceProto for GrpcApi<T>
    where
        T: OrganizationManagementApi + Send + Sync + 'static,
    {
        async fn create_organization(
            &self,
            request: tonic::Request<CreateOrganizationRequestProto>,
        ) -> Result<tonic::Response<CreateOrganizationResponseProto>, tonic::Status> {
            let proto_request = request.into_inner();
            let native_request = proto_request.into();
            let native_response = self
                .api_server
                .create_organization(native_request)
                .await
                .map_err(Into::into)?;

            let proto_response = native_response.try_into().map_err(SerDeError::from)?;

            Ok(tonic::Response::new(proto_response))
        }

        async fn create_user(
            &self,
            request: tonic::Request<CreateUserRequestProto>,
        ) -> Result<tonic::Response<CreateUserResponseProto>, tonic::Status> {
            let proto_request = request.into_inner();
            let native_request = proto_request.try_into()?;
            let native_response = self
                .api_server
                .create_user(native_request)
                .await
                .map_err(Into::into)?;

            let proto_response = native_response.try_into().map_err(SerDeError::from)?;

            Ok(tonic::Response::new(proto_response))
        }
    }

    //
    // public API
    //

    /// Implement this trait to define the organization management API's
    /// business logic
    #[tonic::async_trait]
    pub trait OrganizationManagementApi {
        type Error: Into<Status>;

        async fn create_organization(
            &self,
            request: CreateOrganizationRequest,
        ) -> Result<CreateOrganizationResponse, Self::Error>;

        async fn create_user(
            &self,
            request: CreateUserRequest,
        ) -> Result<CreateUserResponse, Self::Error>;
    }

    /// The organization management server serves the organization management
    /// API
    pub struct OrganizationManagementServer<T, H, F>
    where
        T: OrganizationManagementApi + Send + Sync + 'static,
        H: Fn() -> F + Send + Sync + 'static,
        F: Future<Output = Result<HealthcheckStatus, HealthcheckError>> + Send + Sync + 'static,
    {
        api_server: T,
        healthcheck: H,
        healthcheck_polling_interval: Duration,
        tcp_listener: TcpListener,
        shutdown_rx: Receiver<()>,
        service_name: &'static str,
    }

    impl<T, H, F> OrganizationManagementServer<T, H, F>
    where
        T: OrganizationManagementApi + Send + Sync + 'static,
        H: Fn() -> F + Send + Sync + 'static,
        F: Future<Output = Result<HealthcheckStatus, HealthcheckError>> + Send + Sync + 'static,
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
        ) -> (Self, oneshot::Sender<()>) {
            let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
            (
                OrganizationManagementServer {
                    api_server,
                    healthcheck,
                    healthcheck_polling_interval,
                    tcp_listener,
                    shutdown_rx,
                    service_name: OrganizationManagementServiceServerProto::<GrpcApi<T>>::NAME,
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
            let service_name = &(*self.service_name());
            let (healthcheck_handle, health_service) =
                init_health_service::<OrganizationManagementServiceServerProto<GrpcApi<T>>, _, _>(
                    self.healthcheck,
                    self.healthcheck_polling_interval,
                )
                .await;

            Ok(Server::builder()
                .trace_fn(move |request| {
                    tracing::info_span!(
                        "request_log",
                        service_name = ?service_name,
                        headers = ?request.headers(),
                        method = ?request.method(),
                        uri = %request.uri(),
                        extensions = ?request.extensions(),
                    )
                })
                .add_service(health_service)
                .add_service(OrganizationManagementServiceServerProto::new(GrpcApi::new(
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
