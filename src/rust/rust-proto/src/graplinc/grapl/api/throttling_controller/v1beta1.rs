use crate::{
    graplinc::common::v1beta1::Uuid,
    protobufs::graplinc::grapl::api::throttling_controller::v1beta1::{
        ThrottlingRateForEventSourceRequest as ThrottlingRateForEventSourceRequestProto,
        ThrottlingRateForEventSourceResponse as ThrottlingRateForEventSourceResponseProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

//
// ThrottlingRateForEventSourceRequest
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThrottlingRateForEventSourceRequest {
    event_source_id: Uuid,
}

impl ThrottlingRateForEventSourceRequest {
    pub fn new(event_source_id: Uuid) -> Self {
        Self { event_source_id }
    }

    pub fn event_source_id(&self) -> Uuid {
        self.event_source_id
    }
}

impl TryFrom<ThrottlingRateForEventSourceRequestProto> for ThrottlingRateForEventSourceRequest {
    type Error = SerDeError;

    fn try_from(
        request_proto: ThrottlingRateForEventSourceRequestProto,
    ) -> Result<Self, Self::Error> {
        let event_source_id = request_proto
            .event_source_id
            .ok_or(SerDeError::MissingField("event_source_id"))?
            .into();

        Ok(Self::new(event_source_id))
    }
}

impl From<ThrottlingRateForEventSourceRequest> for ThrottlingRateForEventSourceRequestProto {
    fn from(request: ThrottlingRateForEventSourceRequest) -> Self {
        Self {
            event_source_id: Some(request.event_source_id().into()),
        }
    }
}

impl type_url::TypeUrl for ThrottlingRateForEventSourceRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.throttling_controller.v1beta1.ThrottlingRateForEventSourceRequest";
}

impl serde_impl::ProtobufSerializable for ThrottlingRateForEventSourceRequest {
    type ProtobufMessage = ThrottlingRateForEventSourceRequestProto;
}

//
// ThrottlingRateForEventSourceResponse
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThrottlingRateForEventSourceResponse {
    events_per_second: u32,
}

impl ThrottlingRateForEventSourceResponse {
    pub fn new(events_per_second: u32) -> Self {
        Self { events_per_second }
    }

    pub fn events_per_second(&self) -> u32 {
        self.events_per_second
    }
}

impl From<ThrottlingRateForEventSourceResponseProto> for ThrottlingRateForEventSourceResponse {
    fn from(response_proto: ThrottlingRateForEventSourceResponseProto) -> Self {
        Self::new(response_proto.events_per_second)
    }
}

impl From<ThrottlingRateForEventSourceResponse> for ThrottlingRateForEventSourceResponseProto {
    fn from(response: ThrottlingRateForEventSourceResponse) -> Self {
        Self {
            events_per_second: response.events_per_second(),
        }
    }
}

impl type_url::TypeUrl for ThrottlingRateForEventSourceResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.throttling_controller.v1beta1.ThrottlingRateForEventSourceResponse";
}

impl serde_impl::ProtobufSerializable for ThrottlingRateForEventSourceResponse {
    type ProtobufMessage = ThrottlingRateForEventSourceResponseProto;
}

//
// Client
//

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
            throttling_controller::v1beta1 as native,
        },
        protobufs::graplinc::grapl::api::throttling_controller::v1beta1::throttling_controller_service_client::ThrottlingControllerServiceClient,
    };

    #[async_trait::async_trait]
    impl Connectable for ThrottlingControllerServiceClient<tonic::transport::Channel> {
        async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
            Ok(Self::connect(endpoint).await?)
        }
    }

    #[derive(Clone)]
    pub struct ThrottlingControllerClient {
        client: Client<ThrottlingControllerServiceClient<tonic::transport::Channel>>,
    }

    impl WithClient<ThrottlingControllerServiceClient<tonic::transport::Channel>>
        for ThrottlingControllerClient
    {
        fn with_client(
            client: Client<ThrottlingControllerServiceClient<tonic::transport::Channel>>,
        ) -> Self {
            Self { client }
        }
    }

    impl ThrottlingControllerClient {
        pub async fn throttling_rate_for_event_source(
            &mut self,
            request: native::ThrottlingRateForEventSourceRequest,
        ) -> Result<native::ThrottlingRateForEventSourceResponse, ClientError> {
            self.client
                .execute(
                    request,
                    |status| status.code() == tonic::Code::Unavailable,
                    10,
                    |mut client, request| async move {
                        client.throttling_rate_for_event_source(request).await
                    },
                )
                .await
        }
    }
}

//
// Server
//

pub mod server {
    use std::time::Duration;

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
    use tonic::transport::{
        NamedService,
        Server,
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
            throttling_controller::v1beta1::{
                ThrottlingRateForEventSourceRequest,
                ThrottlingRateForEventSourceResponse,
            },
        },
        protobufs::graplinc::grapl::api::throttling_controller::v1beta1::{
            throttling_controller_service_server::{
                ThrottlingControllerService as ThrottlingControllerServiceProto,
                ThrottlingControllerServiceServer as ThrottlingControllerServiceServerProto,
            },
            ThrottlingRateForEventSourceRequest as ThrottlingRateForEventSourceRequestProto,
            ThrottlingRateForEventSourceResponse as ThrottlingRateForEventSourceResponseProto,
        },
    };

    //
    // protocol buffer stuff
    //

    #[tonic::async_trait]
    impl<T> ThrottlingControllerServiceProto for GrpcApi<T>
    where
        T: ThrottlingControllerApi + Send + Sync + 'static,
    {
        async fn throttling_rate_for_event_source(
            &self,
            request: tonic::Request<ThrottlingRateForEventSourceRequestProto>,
        ) -> Result<tonic::Response<ThrottlingRateForEventSourceResponseProto>, tonic::Status>
        {
            execute_rpc!(self, request, throttling_rate_for_event_source)
        }
    }

    //
    // public API
    //

    #[tonic::async_trait]
    pub trait ThrottlingControllerApi {
        type Error: Into<Status>;

        async fn throttling_rate_for_event_source(
            &self,
            request: ThrottlingRateForEventSourceRequest,
        ) -> Result<ThrottlingRateForEventSourceResponse, Self::Error>;
    }

    pub struct ThrottlingControllerServer<T, H, F>
    where
        T: ThrottlingControllerApi + Send + Sync + 'static,
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

    impl<T, H, F> ThrottlingControllerServer<T, H, F>
    where
        T: ThrottlingControllerApi + Send + Sync + 'static,
        H: Fn() -> F + Send + Sync + 'static,
        F: Future<Output = Result<HealthcheckStatus, HealthcheckError>> + Send + Sync + 'static,
    {
        pub fn new(
            api_server: T,
            tcp_listener: TcpListener,
            healthcheck: H,
            healthcheck_polling_interval: Duration,
        ) -> (Self, Sender<()>) {
            let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
            (
                ThrottlingControllerServer {
                    api_server,
                    healthcheck,
                    healthcheck_polling_interval,
                    tcp_listener,
                    shutdown_rx,
                    service_name: ThrottlingControllerServiceServerProto::<GrpcApi<T>>::NAME,
                },
                shutdown_tx,
            )
        }

        pub fn service_name(&self) -> &'static str {
            self.service_name
        }

        pub async fn serve(self) -> Result<(), ServeError> {
            let (healthcheck_handle, health_service) =
                init_health_service::<ThrottlingControllerServiceServerProto<GrpcApi<T>>, _, _>(
                    self.healthcheck,
                    self.healthcheck_polling_interval,
                )
                .await;

            Ok(Server::builder()
                .trace_fn(|_request| tracing::info_span!("throttling-controller"))
                .add_service(health_service)
                .add_service(ThrottlingControllerServiceServerProto::new(GrpcApi::new(
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
