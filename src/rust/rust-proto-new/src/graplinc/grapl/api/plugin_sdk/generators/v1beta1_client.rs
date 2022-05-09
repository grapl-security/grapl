use std::fmt::Debug;

use generator_service_client::GeneratorServiceClient as GeneratorServiceClientProto;

pub use crate::protobufs::graplinc::grapl::api::plugin_sdk::generators::v1beta1::generator_service_client;
use crate::{
    graplinc::grapl::api::plugin_sdk::generators::v1beta1 as native,
    protobufs::graplinc::grapl::api::plugin_sdk::generators::v1beta1 as proto,
    SerDeError,
};

#[derive(Debug, thiserror::Error)]
pub enum GeneratorServiceClientError {
    #[error("ErrorStatus")]
    ErrorStatus(#[from] tonic::Status),
    #[error("PluginRegistryDeserializationError")]
    GeneratorDeserializationError(#[from] SerDeError),
}

pub struct GeneratorServiceClient {
    proto_client: GeneratorServiceClientProto<tonic::transport::Channel>,
}

impl GeneratorServiceClient {
    #[tracing::instrument(err)]
    pub async fn connect<T>(endpoint: T) -> Result<Self, Box<dyn std::error::Error>>
    where
        T: std::convert::TryInto<tonic::transport::Endpoint> + Debug,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        Ok(Self {
            proto_client: GeneratorServiceClientProto::connect(endpoint).await?,
        })
    }

    pub async fn run_generator(
        &mut self,
        request: native::RunGeneratorRequest,
    ) -> Result<native::RunGeneratorResponse, GeneratorServiceClientError> {
        let response = self
            .proto_client
            .run_generator(proto::RunGeneratorRequest::from(request))
            .await?;
        let response = native::RunGeneratorResponse::try_from(response.into_inner())?;
        Ok(response)
    }
}
