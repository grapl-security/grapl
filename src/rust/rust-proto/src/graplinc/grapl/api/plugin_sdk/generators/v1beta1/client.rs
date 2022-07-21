use std::fmt::Debug;

use generator_service_client::GeneratorServiceClient as GeneratorServiceClientProto;

pub use crate::protobufs::graplinc::grapl::api::plugin_sdk::generators::v1beta1::generator_service_client;
use crate::{
    graplinc::grapl::api::plugin_sdk::generators::v1beta1 as native,
    protobufs::graplinc::grapl::api::plugin_sdk::generators::v1beta1 as proto,
    protocol::{
        endpoint::Endpoint,
        service_client::{
            ConnectError,
            Connectable,
            NamedService,
        },
        status::Status,
    },
    SerDeError,
};

#[derive(Debug, thiserror::Error)]
pub enum GeneratorServiceClientError {
    #[error("ErrorStatus")]
    ErrorStatus(#[from] Status),
    #[error("PluginRegistryDeserializationError")]
    GeneratorDeserializationError(#[from] SerDeError),
}

#[derive(Clone)]
pub struct GeneratorServiceClient {
    proto_client: GeneratorServiceClientProto<tonic::transport::Channel>,
}
#[async_trait::async_trait]
impl Connectable for GeneratorServiceClient {
    #[tracing::instrument(err)]
    async fn connect(endpoint: Endpoint) -> Result<Self, ConnectError> {
        Ok(GeneratorServiceClient {
            proto_client: GeneratorServiceClientProto::connect(endpoint).await?,
        })
    }
}

impl GeneratorServiceClient {
    pub async fn run_generator(
        &mut self,
        request: native::RunGeneratorRequest,
    ) -> Result<native::RunGeneratorResponse, GeneratorServiceClientError> {
        let response = self
            .proto_client
            .run_generator(proto::RunGeneratorRequest::from(request))
            .await
            .map_err(Status::from)?;
        let response = native::RunGeneratorResponse::try_from(response.into_inner())?;
        Ok(response)
    }
}

impl NamedService for GeneratorServiceClient {
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.plugin_sdk.generators.v1beta1.GeneratorService";
}
