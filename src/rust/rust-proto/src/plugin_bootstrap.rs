use std::fmt::Formatter;
pub use crate::graplinc::grapl::api::plugin_bootstrap::v1beta1::{
    GetBootstrapInfoRequest as GetBootstrapInfoRequestProto,
    GetBootstrapInfoResponse as GetBootstrapInfoResponseProto,
    plugin_bootstrap_service_client::PluginBootstrapServiceClient,
    plugin_bootstrap_service_server::{PluginBootstrapServiceServer, PluginBootstrapService},
};


#[derive(Debug, thiserror::Error)]
pub enum PluginBootstrapDeserializationError {
    #[error("Missing a required field")]
    MissingRequiredField(&'static str),
    #[error("Empty field")]
    EmptyField(&'static str),
    #[error("Unknown variant")]
    UnknownVariant(std::borrow::Cow<'static, str>),
}

#[derive(Clone, Debug)]
pub struct GetBootstrapInfoRequest {
}


impl TryFrom<GetBootstrapInfoRequestProto> for GetBootstrapInfoRequest {
    type Error = PluginBootstrapDeserializationError;

    fn try_from(_: GetBootstrapInfoRequestProto) -> Result<Self, Self::Error> {
        Ok(GetBootstrapInfoRequest {})
    }
}

impl From<GetBootstrapInfoRequest> for GetBootstrapInfoRequestProto {
    fn from(_: GetBootstrapInfoRequest) -> Self {
        GetBootstrapInfoRequestProto {}
    }
}

#[derive(Clone)]
pub struct GetBootstrapInfoResponse {
    pub plugin_binary: Vec<u8>,
    pub certificate: Vec<u8>,
}

impl std::fmt::Debug for GetBootstrapInfoResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GetBootstrapInfoResponse")
            .field("plugin_binary.len", &self.plugin_binary.len())
            .field("certificate.len", &self.certificate.len())
            .finish()
    }
}

impl TryFrom<GetBootstrapInfoResponseProto> for GetBootstrapInfoResponse {
    type Error = PluginBootstrapDeserializationError;

    fn try_from(value: GetBootstrapInfoResponseProto) -> Result<Self, Self::Error> {
        if value.plugin_binary.is_empty() {
            return Err(PluginBootstrapDeserializationError::EmptyField("GetBootstrapInfoResponseProto.plugin_binary"))
        }

        if value.certificate.is_empty() {
            return Err(PluginBootstrapDeserializationError::EmptyField("GetBootstrapInfoResponseProto.certificate"))
        }

        Ok(GetBootstrapInfoResponse {
            plugin_binary: value.plugin_binary,
            certificate: value.certificate,
        })
    }
}

impl From<GetBootstrapInfoResponse> for GetBootstrapInfoResponseProto {
    fn from(value: GetBootstrapInfoResponse) -> Self {
        GetBootstrapInfoResponseProto { plugin_binary: value.plugin_binary, certificate: value.certificate }
    }
}