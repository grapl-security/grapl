use bytes::Bytes;

use crate::{
    protobufs::graplinc::grapl::api::plugin_bootstrap::v1beta1::{
        ClientCertificate as ClientCertificateProto,
        GetBootstrapRequest as GetBootstrapRequestProto,
        GetBootstrapResponse as GetBootstrapResponseProto,
        PluginPayload as PluginPayloadProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

//
// ClientCertificate
//

#[derive(Clone)]
pub struct ClientCertificate {
    pub client_certificate: Bytes,
}

impl From<ClientCertificateProto> for ClientCertificate {
    fn from(client_certificate_proto: ClientCertificateProto) -> Self {
        ClientCertificate {
            client_certificate: client_certificate_proto.client_certificate,
        }
    }
}

impl From<ClientCertificate> for ClientCertificateProto {
    fn from(client_certificate: ClientCertificate) -> Self {
        ClientCertificateProto {
            client_certificate: client_certificate.client_certificate,
        }
    }
}

impl type_url::TypeUrl for ClientCertificate {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_bootstrap.v1beta1.ClientCertificate";
}

impl serde_impl::ProtobufSerializable for ClientCertificate {
    type ProtobufMessage = ClientCertificateProto;
}

//
// GetBootstrapRequest
//

pub struct GetBootstrapRequest {
    // empty
}

impl From<GetBootstrapRequestProto> for GetBootstrapRequest {
    fn from(_get_bootstrap_request_proto: GetBootstrapRequestProto) -> Self {
        GetBootstrapRequest {}
    }
}

impl From<GetBootstrapRequest> for GetBootstrapRequestProto {
    fn from(_get_bootstrap_request: GetBootstrapRequest) -> Self {
        GetBootstrapRequestProto {}
    }
}

impl type_url::TypeUrl for GetBootstrapRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_bootstrap.v1beta1.GetBootstrapRequest";
}

impl serde_impl::ProtobufSerializable for GetBootstrapRequest {
    type ProtobufMessage = GetBootstrapRequestProto;
}

//
// GetBootstrapResponse
//

pub struct GetBootstrapResponse {
    pub plugin_payload: PluginPayload,
    pub client_certificate: ClientCertificate,
}

impl TryFrom<GetBootstrapResponseProto> for GetBootstrapResponse {
    type Error = SerDeError;

    fn try_from(
        get_bootstrap_response_proto: GetBootstrapResponseProto,
    ) -> Result<Self, Self::Error> {
        let client_certificate = get_bootstrap_response_proto
            .client_certificate
            .ok_or(SerDeError::MissingField("client_certificate"))?;
        let plugin_payload = get_bootstrap_response_proto
            .plugin_payload
            .ok_or(SerDeError::MissingField("plugin_payload"))?;

        Ok(GetBootstrapResponse {
            plugin_payload: plugin_payload.into(),
            client_certificate: client_certificate.into(),
        })
    }
}

impl From<GetBootstrapResponse> for GetBootstrapResponseProto {
    fn from(get_bootstrap_response: GetBootstrapResponse) -> Self {
        GetBootstrapResponseProto {
            plugin_payload: Some(get_bootstrap_response.plugin_payload.into()),
            client_certificate: Some(get_bootstrap_response.client_certificate.into()),
        }
    }
}

impl type_url::TypeUrl for GetBootstrapResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_bootstrap.v1beta1.GetBootstrapResponse";
}

impl serde_impl::ProtobufSerializable for GetBootstrapResponse {
    type ProtobufMessage = GetBootstrapResponseProto;
}

//
// PluginPayload
//

#[derive(Clone)]
pub struct PluginPayload {
    pub plugin_binary: Bytes,
}

impl From<PluginPayloadProto> for PluginPayload {
    fn from(plugin_payload_proto: PluginPayloadProto) -> Self {
        PluginPayload {
            plugin_binary: plugin_payload_proto.plugin_binary,
        }
    }
}

impl From<PluginPayload> for PluginPayloadProto {
    fn from(plugin_payload: PluginPayload) -> Self {
        PluginPayloadProto {
            plugin_binary: plugin_payload.plugin_binary,
        }
    }
}

impl type_url::TypeUrl for PluginPayload {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_bootstrap.v1beta1.PluginPayload";
}

impl serde_impl::ProtobufSerializable for PluginPayload {
    type ProtobufMessage = PluginPayloadProto;
}

//
// client
//

pub mod client {
    use futures::FutureExt;
    use thiserror::Error;
    use tokio::time::error::Elapsed;
    use tonic::Request;

    use crate::{
        protobufs::graplinc::grapl::api::plugin_bootstrap::v1beta1::plugin_bootstrap_service_client::PluginBootstrapServiceClient as PluginBootstrapServiceClientProto,
        protocol::healthcheck::HealthcheckError,
        SerDeError,
    };

    use super::{
        GetBootstrapRequest,
        GetBootstrapResponse,
    };

    #[non_exhaustive]
    #[derive(Debug, Error)]
    pub enum ConfigurationError {
        #[error("failed to connect {0}")]
        ConnectionError(#[from] tonic::transport::Error),

        #[error("healthcheck failed {0}")]
        HealtcheckFailed(#[from] HealthcheckError),

        #[error("timeout elapsed {0}")]
        TimeoutElapsed(#[from] Elapsed),
    }

    #[non_exhaustive]
    #[derive(Debug, Error)]
    pub enum PluginBootstrapClientError {
        #[error("failed to serialize/deserialize {0}")]
        SerDeError(#[from] SerDeError),

        #[error("received unfavorable gRPC status {0}")]
        GrpcStatus(#[from] tonic::Status),
    }

    pub struct PluginBootstrapClient {
        proto_client: PluginBootstrapServiceClientProto<tonic::transport::Channel>,
    }

    impl PluginBootstrapClient {
        pub async fn connect<T>(endpoint: T) -> Result<Self, ConfigurationError>
        where
            T: std::convert::TryInto<tonic::transport::Endpoint>,
            T::Error: std::error::Error + Send + Sync + 'static,
        {
            Ok(PluginBootstrapClient {
                proto_client: PluginBootstrapServiceClientProto::connect(endpoint).await?,
            })
        }

        pub async fn get_bootstrap(
            &mut self,
            get_bootstrap_request: GetBootstrapRequest,
        ) -> Result<GetBootstrapResponse, PluginBootstrapClientError> {
            self.proto_client
                .get_bootstrap(Request::new(get_bootstrap_request.into()))
                .map(
                    |response| -> Result<GetBootstrapResponse, PluginBootstrapClientError> {
                        let inner = response?.into_inner();
                        Ok(inner.try_into()?)
                    },
                )
                .await
        }
    }
}

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
        GetBootstrapRequest,
        GetBootstrapResponse,
    };
    use crate::{
        protobufs::graplinc::grapl::api::plugin_bootstrap::v1beta1::{
            plugin_bootstrap_service_server::{
                PluginBootstrapService as PluginBootstrapServiceProto,
                PluginBootstrapServiceServer as PluginBootstrapServiceServerProto,
            },
            GetBootstrapRequest as GetBootstrapRequestProto,
            GetBootstrapResponse as GetBootstrapResponseProto,
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
    impl<T> PluginBootstrapServiceProto for GrpcApi<T>
    where
        T: PluginBootstrapApi + Send + Sync + 'static,
    {
        async fn get_bootstrap(
            &self,
            request: tonic::Request<GetBootstrapRequestProto>,
        ) -> Result<tonic::Response<GetBootstrapResponseProto>, tonic::Status> {
            let proto_request = request.into_inner();
            let native_request = proto_request.into();
            let native_response = self
                .api_server
                .get_bootstrap(native_request)
                .await
                .map_err(Into::into)?;

            let proto_response = native_response.try_into().map_err(SerDeError::from)?;

            Ok(tonic::Response::new(proto_response))
        }
    }

    //
    // public API
    //

    /// Implement this trait to define the plugin bootstrap API's business logic
    #[tonic::async_trait]
    pub trait PluginBootstrapApi {
        type Error: Into<Status>;

        async fn get_bootstrap(
            &self,
            request: GetBootstrapRequest,
        ) -> Result<GetBootstrapResponse, Self::Error>;
    }

    /// The plugin bootstrap server serves the plugin bootstrap API
    pub struct PluginBootstrapServer<T, H, F>
    where
        T: PluginBootstrapApi + Send + Sync + 'static,
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

    impl<T, H, F> PluginBootstrapServer<T, H, F>
    where
        T: PluginBootstrapApi + Send + Sync + 'static,
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
                PluginBootstrapServer {
                    api_server,
                    healthcheck,
                    healthcheck_polling_interval,
                    tcp_listener,
                    shutdown_rx,
                    service_name: PluginBootstrapServiceServerProto::<GrpcApi<T>>::NAME,
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
                init_health_service::<PluginBootstrapServiceServerProto<GrpcApi<T>>, _, _>(
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
                .add_service(PluginBootstrapServiceServerProto::new(GrpcApi::new(
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
