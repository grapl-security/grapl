use std::{
    convert::Infallible,
    fmt::Debug,
    time::Duration,
};

use client_executor::{
    Executor,
    ExecutorConfig,
};
use generator_service_client::GeneratorServiceClient as GeneratorServiceClientProto;

pub use crate::protobufs::graplinc::grapl::api::plugin_sdk::generators::v1beta1::generator_service_client;
use crate::{
    execute_client_rpc,
    get_proto_client,
    graplinc::grapl::api::plugin_sdk::generators::v1beta1 as native,
    protobufs::graplinc::grapl::api::plugin_sdk::generators::v1beta1 as proto,
    protocol::{
        endpoint::Endpoint,
        service_client::{
            ConnectError,
            Connectable,
        },
        status::Status,
    },
    SerDeError,
};

// TODO It looks like *ClientError is basically duplicated everywhere, we could
// simplify and have GrpcClientError or something
#[derive(Debug, thiserror::Error)]
pub enum GeneratorServiceClientError {
    #[error("ErrorStatus")]
    ErrorStatus(#[from] Status),
    #[error("PluginRegistryDeserializationError")]
    GeneratorDeserializationError(#[from] SerDeError),
    #[error("CircuitOpen")]
    CircuitOpen,
    #[error("Timeout")]
    Elapsed,
}

// A compatibility layer for using
// TryFrom<Error = SerDeError>
// in place of From.
impl From<Infallible> for GeneratorServiceClientError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<client_executor::Error<tonic::Status>> for GeneratorServiceClientError {
    fn from(e: client_executor::Error<tonic::Status>) -> Self {
        match e {
            client_executor::Error::Inner(e) => Self::ErrorStatus(e.into()),
            client_executor::Error::Rejected => Self::CircuitOpen,
            client_executor::Error::Elapsed => Self::Elapsed,
        }
    }
}

#[derive(Clone)]
pub struct GeneratorServiceClient {
    executor: Executor,
    proto_client: GeneratorServiceClientProto<tonic::transport::Channel>,
}
#[async_trait::async_trait]
impl Connectable for GeneratorServiceClient {
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.plugin_sdk.generators.v1beta1.GeneratorService";

    #[tracing::instrument(err)]
    async fn connect(endpoint: Endpoint) -> Result<Self, ConnectError> {
        let executor = Executor::new(ExecutorConfig::new(Duration::from_secs(30)));
        let proto_client = get_proto_client!(
            executor,
            GeneratorServiceClientProto<tonic::transport::Channel>,
            endpoint,
        );
        Ok(GeneratorServiceClient {
            executor,
            proto_client,
        })
    }
}

impl GeneratorServiceClient {
    pub async fn run_generator(
        &mut self,
        request: native::RunGeneratorRequest,
    ) -> Result<native::RunGeneratorResponse, GeneratorServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            run_generator,
            proto::RunGeneratorRequest,
            native::RunGeneratorResponse,
        )
    }
}
