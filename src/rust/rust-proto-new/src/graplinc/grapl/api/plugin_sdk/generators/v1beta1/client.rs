use std::fmt::Debug;

use generator_service_client::GeneratorServiceClient as GeneratorServiceClientProto;

pub use crate::protobufs::graplinc::grapl::api::plugin_sdk::generators::v1beta1::generator_service_client;
use crate::{
    graplinc::grapl::api::plugin_sdk::generators::v1beta1 as native,
    protobufs::graplinc::grapl::api::plugin_sdk::generators::v1beta1 as proto,
    protocol::{
        status::Status,
        tls::ClientTlsConfig,
    },
    SerDeError,
};

#[derive(Debug, thiserror::Error)]
pub enum GeneratorServiceClientError {
    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),
    #[error("ErrorStatus")]
    ErrorStatus(#[from] Status),
    #[error("PluginRegistryDeserializationError")]
    GeneratorDeserializationError(#[from] SerDeError),
}

#[derive(Clone)]
pub struct GeneratorServiceClient {
    proto_client: GeneratorServiceClientProto<tonic::transport::Channel>,
}

impl GeneratorServiceClient {
    #[tracing::instrument(skip(tls_config), err)]
    pub async fn connect<T>(
        endpoint: T,
        tls_config: Option<ClientTlsConfig>,
    ) -> Result<Self, GeneratorServiceClientError>
    where
        T: std::convert::TryInto<tonic::transport::Endpoint, Error = tonic::transport::Error>
            + Debug,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        let mut endpoint: tonic::transport::Endpoint = endpoint.try_into()?;
        if let Some(inner_config) = tls_config {
            endpoint = endpoint.tls_config(inner_config.into())?;
        }
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
            .await
            .map_err(Status::from)?;
        let response = native::RunGeneratorResponse::try_from(response.into_inner())?;
        Ok(response)
    }
}
