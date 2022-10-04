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
    use std::time::Duration;

    use client_executor::strategy::FibonacciBackoff;
    use tonic::transport::Endpoint;

    use crate::{
        protobufs::graplinc::grapl::api::plugin_bootstrap::v1beta1::plugin_bootstrap_service_client::PluginBootstrapServiceClient, graplinc::grapl::api::client::{Connectable, ClientError, Configuration, Client},
    };

    use super::{
        GetBootstrapRequest,
        GetBootstrapResponse
    };

    #[async_trait::async_trait]
    impl Connectable for PluginBootstrapServiceClient<tonic::transport::Channel> {
        async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
            Ok(Self::connect(endpoint).await?)
        }
    }

    pub struct PluginBootstrapClient<B>
    where
        B: IntoIterator<Item = Duration> + Clone,
    {
        client: Client<B, tonic::transport::Channel>,
    }

    impl <B> PluginBootstrapClient<B>
    where
        B: IntoIterator<Item = Duration> + Clone,
    {
        const SERVICE_NAME: &'static str =
            "graplinc.grapl.api.plugin_bootstrap.v1beta1.PluginBootstrapService";

        pub fn new<A>(
            address: A,
            request_timeout: Duration,
            executor_timeout: Duration,
            concurrency_limit: usize,
            initial_backoff_delay: Duration,
            maximum_backoff_delay: Duration,
        ) -> Result<Self, ClientError>
        where
            A: TryInto<Endpoint>,
        {
            let configuration = Configuration::new(
                Self::SERVICE_NAME,
                address,
                request_timeout,
                executor_timeout,
                concurrency_limit,
                FibonacciBackoff::from_millis(initial_backoff_delay.as_millis())
                    .max_delay(maximum_backoff_delay)
                    .map(client_executor::strategy::jitter),
            )?;
            let client = Client::new(configuration);

            Ok(Self { client })
        }

        pub async fn get_bootstrap(
            &mut self,
            request: GetBootstrapRequest,
        ) -> Result<GetBootstrapResponse, ClientError> {
            Ok(self.client.execute(
                request,
                |status, request| status.code() == tonic::Code::Unavailable,
                10,
                |client, request| client.get_bootstrap(request),
            ).await?)
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
        },
        protobufs::graplinc::grapl::api::plugin_bootstrap::v1beta1::{
            plugin_bootstrap_service_server::{
                PluginBootstrapService as PluginBootstrapServiceProto,
                PluginBootstrapServiceServer as PluginBootstrapServiceServerProto,
            },
            GetBootstrapRequest as GetBootstrapRequestProto,
            GetBootstrapResponse as GetBootstrapResponseProto,
        },
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
            let service_name = self.service_name();
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
