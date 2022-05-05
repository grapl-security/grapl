use bytes::Bytes;

use crate::{
    graplinc::common::v1beta1::{
        SystemTime,
        Uuid,
    },
    protobufs::graplinc::grapl::api::pipeline_ingress::v1beta1::{
        PublishRawLogRequest as PublishRawLogRequestProto,
        PublishRawLogResponse as PublishRawLogResponseProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

//
// PublishRawLogRequest
//

#[derive(Debug, Clone, PartialEq)]
pub struct PublishRawLogRequest {
    pub event_source_id: Uuid,
    pub tenant_id: Uuid,
    pub log_event: Bytes,
}

impl TryFrom<PublishRawLogRequestProto> for PublishRawLogRequest {
    type Error = SerDeError;

    fn try_from(request_proto: PublishRawLogRequestProto) -> Result<Self, Self::Error> {
        let event_source_id = request_proto
            .event_source_id
            .ok_or(SerDeError::MissingField("event_source_id"))?;

        let tenant_id = request_proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?;

        Ok(PublishRawLogRequest {
            event_source_id: event_source_id.into(),
            tenant_id: tenant_id.into(),
            log_event: Bytes::from(request_proto.log_event),
        })
    }
}

impl From<PublishRawLogRequest> for PublishRawLogRequestProto {
    fn from(request: PublishRawLogRequest) -> Self {
        PublishRawLogRequestProto {
            event_source_id: Some(request.event_source_id.into()),
            tenant_id: Some(request.tenant_id.into()),
            log_event: request.log_event.to_vec(),
        }
    }
}

impl type_url::TypeUrl for PublishRawLogRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.pipeline_ingress.v1beta1.PublishRawLogRequest";
}

impl serde_impl::ProtobufSerializable<PublishRawLogRequest> for PublishRawLogRequest {
    type ProtobufMessage = PublishRawLogRequestProto;
}

//
// PublishRawLogResponse
//

#[derive(Debug, Clone, PartialEq)]
pub struct PublishRawLogResponse {
    pub created_time: SystemTime,
}

impl PublishRawLogResponse {
    /// build a response with created_time set to SystemTime::now()
    pub fn ok() -> Self {
        PublishRawLogResponse {
            created_time: SystemTime::now(),
        }
    }
}

impl TryFrom<PublishRawLogResponseProto> for PublishRawLogResponse {
    type Error = SerDeError;

    fn try_from(response_proto: PublishRawLogResponseProto) -> Result<Self, Self::Error> {
        let created_time = response_proto
            .created_time
            .ok_or(SerDeError::MissingField("created_time"))?;

        Ok(PublishRawLogResponse {
            created_time: created_time.try_into()?,
        })
    }
}

impl TryFrom<PublishRawLogResponse> for PublishRawLogResponseProto {
    type Error = SerDeError;

    fn try_from(response: PublishRawLogResponse) -> Result<Self, Self::Error> {
        Ok(PublishRawLogResponseProto {
            created_time: Some(response.created_time.try_into()?),
        })
    }
}

impl type_url::TypeUrl for PublishRawLogResponse {
    const TYPE_URL: &'static str =
        "grapsecurity.com/graplinc.grapl.api.pipeline_ingress.v1beta1.PublishRawLogResponse";
}

impl serde_impl::ProtobufSerializable<PublishRawLogResponse> for PublishRawLogResponse {
    type ProtobufMessage = PublishRawLogResponseProto;
}

//
// client
//

/// This module contains the gRPC client for the pipeline ingress API. We
/// encapsulate all the types generated by the protocol buffer compiler and
/// instead expose our own "sanitized" version of the API.
pub mod client {
    use crate::{
        graplinc::grapl::api::pipeline_ingress::v1beta1::{
            PublishRawLogRequest,
            PublishRawLogResponse,
        },
        protobufs::graplinc::grapl::api::pipeline_ingress::v1beta1::pipeline_ingress_service_client::PipelineIngressServiceClient as PipelineIngressServiceClientProto,
        SerDeError,
    };
    use crate::protocol::healthcheck::HealthcheckError;

    use futures::FutureExt;
    use thiserror::Error;
    use tokio::time::error::Elapsed;
    use tonic::Request;
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
    pub enum PipelineIngressApiError {
        #[error("failed to serialize/deserialize {0}")]
        SerDeError(#[from] SerDeError),

        #[error("received unfavorable gRPC status {0}")]
        GrpcStatus(#[from] tonic::Status),
    }

    pub struct PipelineIngressClient {
        proto_client: PipelineIngressServiceClientProto<tonic::transport::Channel>,
    }

    impl PipelineIngressClient {
        pub async fn connect<T>(endpoint: T) -> Result<Self, ConfigurationError>
        where
            T: std::convert::TryInto<tonic::transport::Endpoint>,
            T::Error: std::error::Error + Send + Sync + 'static,
        {
            Ok(PipelineIngressClient {
                proto_client: PipelineIngressServiceClientProto::connect(endpoint).await?,
            })
        }

        pub async fn publish_raw_log(
            &mut self,
            raw_log: PublishRawLogRequest,
        ) -> Result<PublishRawLogResponse, PipelineIngressApiError> {
            self.proto_client
                .publish_raw_log(Request::new(raw_log.into()))
                .map(
                    |response| -> Result<PublishRawLogResponse, PipelineIngressApiError> {
                        let inner = response?.into_inner();
                        Ok(inner.try_into()?)
                    },
                )
                .await
        }
    }
}

//
// server
//

/// This module contains the gRPC server for serving the pipeline ingress
/// API. We encapsulate all the types generated by the protocol buffer compiler
/// and instead expose our own "sanitized" version of the API. Users should
/// implement their business logic by constructing an object conforming to the
/// PipelineIngressApi trait. To run the gRPC server implementing that business
/// logic, inject the PipelineIngressApi implementation into the
/// PipelineIngressServer's constructor.
pub mod server {
    use std::{
        marker::PhantomData,
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
    };
    use thiserror::Error;
    use tokio::net::TcpListener;
    use tokio_stream::wrappers::TcpListenerStream;
    use tonic::transport::{
        NamedService,
        Server,
    };

    use crate::{
        graplinc::grapl::api::pipeline_ingress::v1beta1::{
            PublishRawLogRequest,
            PublishRawLogResponse,
        },
        protobufs::graplinc::grapl::api::pipeline_ingress::v1beta1::{
            pipeline_ingress_service_server::{
                PipelineIngressService as PipelineIngressServiceProto,
                PipelineIngressServiceServer as PipelineIngressServiceServerProto,
            },
            PublishRawLogRequest as PublishRawLogRequestProto,
            PublishRawLogResponse as PublishRawLogResponseProto,
        },
        protocol::{
            healthcheck::{
                server::init_health_service,
                HealthcheckError,
                HealthcheckStatus,
            },
            status::Status,
        },
        rpc_translate_proto_to_native,
        server_internals::ApiDelegate,
        SerDeError,
    };

    //
    // protocol buffer stuff
    //

    #[tonic::async_trait]
    impl<T> PipelineIngressServiceProto for ApiDelegate<T>
    where
        T: PipelineIngressApi + Send + Sync + 'static,
    {
        async fn publish_raw_log(
            &self,
            request: tonic::Request<PublishRawLogRequestProto>,
        ) -> Result<tonic::Response<PublishRawLogResponseProto>, tonic::Status> {
            rpc_translate_proto_to_native!(self, request, publish_raw_log)
        }
    }

    //
    // public API
    //

    /// Implement this trait to define the pipeline ingress API's business logic
    #[tonic::async_trait]
    pub trait PipelineIngressApi {
        type Error: Into<Status>;
        async fn publish_raw_log(
            &self,
            request: PublishRawLogRequest,
        ) -> Result<PublishRawLogResponse, Self::Error>;
    }

    #[non_exhaustive]
    #[derive(Debug, Error)]
    pub enum ConfigurationError {
        #[error("encountered tonic error {0}")]
        TonicError(#[from] tonic::transport::Error),
    }

    /// The pipeline-ingress server serves the pipeline-ingress API
    pub struct PipelineIngressServer<T, H, F>
    where
        T: PipelineIngressApi + Send + Sync + 'static,
        H: Fn() -> F + Send + Sync + 'static,
        F: Future<Output = Result<HealthcheckStatus, HealthcheckError>> + Send + Sync + 'static,
    {
        api_server: T,
        healthcheck: H,
        healthcheck_polling_interval: Duration,
        tcp_listener: TcpListener,
        shutdown_rx: Receiver<()>,
        service_name: &'static str,
        f_: PhantomData<F>,
    }

    impl<T, H, F> PipelineIngressServer<T, H, F>
    where
        T: PipelineIngressApi + Send + Sync + 'static,
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
        ) -> (Self, Sender<()>) {
            let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
            (
                PipelineIngressServer {
                    api_server,
                    healthcheck,
                    healthcheck_polling_interval,
                    tcp_listener,
                    shutdown_rx,
                    service_name: PipelineIngressServiceServerProto::<ApiDelegate<T>>::NAME,
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
        /// address. Returns a ConfigurationError if the gRPC server cannot run.
        pub async fn serve(self) -> Result<(), ConfigurationError> {
            // TODO: add tower tracing, tls_config, concurrency limits
            let (healthcheck_handle, health_service) =
                init_health_service::<PipelineIngressServiceServerProto<ApiDelegate<T>>, _, _>(
                    self.healthcheck,
                    self.healthcheck_polling_interval,
                )
                .await;
            Ok(Server::builder()
                .add_service(health_service)
                .add_service(PipelineIngressServiceServerProto::new(ApiDelegate::new(
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
