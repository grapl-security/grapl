use std::fmt::Formatter;

pub use crate::graplinc::grapl::api::plugin_bootstrap::v1beta1::{
    plugin_bootstrap_service_client::PluginBootstrapServiceClient,
    plugin_bootstrap_service_server::{
        PluginBootstrapService,
        PluginBootstrapServiceServer,
    },
    CertificateEncoding as CertificateEncodingProto,
    ClientCertificate as ClientCertificateProto,
    GetBootstrapInfoRequest as GetBootstrapInfoRequestProto,
    GetBootstrapInfoResponse as GetBootstrapInfoResponseProto,
    PluginPayload as PluginPayloadProto,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginBootstrapDeserializationError {
    #[error("Missing a required field {0}")]
    MissingRequiredField(&'static str),
    #[error("Empty field {0}")]
    EmptyField(&'static str),
    #[error("Unknown variant {0}")]
    UnknownVariant(std::borrow::Cow<'static, str>),
}

#[derive(Clone, Debug)]
pub struct GetBootstrapInfoRequest {}

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
pub struct PluginPayload {
    pub plugin_binary: Vec<u8>,
}

impl std::fmt::Debug for PluginPayload {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginPayload")
            .field("plugin_binary.len", &self.plugin_binary.len())
            .finish()
    }
}

impl TryFrom<PluginPayloadProto> for PluginPayload {
    type Error = PluginBootstrapDeserializationError;

    fn try_from(value: PluginPayloadProto) -> Result<Self, Self::Error> {
        if value.plugin_binary.is_empty() {
            return Err(PluginBootstrapDeserializationError::EmptyField(
                "PluginPayload.plugin_binary",
            ));
        }
        Ok(PluginPayload {
            plugin_binary: value.plugin_binary,
        })
    }
}

impl From<PluginPayload> for PluginPayloadProto {
    fn from(value: PluginPayload) -> Self {
        PluginPayloadProto {
            plugin_binary: value.plugin_binary,
        }
    }
}

#[derive(Clone, Debug)]
pub enum CertificateEncoding {
    Pem,
}

impl TryFrom<CertificateEncodingProto> for CertificateEncoding {
    type Error = PluginBootstrapDeserializationError;

    fn try_from(value: CertificateEncodingProto) -> Result<Self, Self::Error> {
        match value {
            CertificateEncodingProto::Unknown => {
                Err(PluginBootstrapDeserializationError::UnknownVariant(
                    "CertificateEncodingProto".into(),
                ))
            }
            CertificateEncodingProto::Pem => Ok(CertificateEncoding::Pem),
        }
    }
}

impl From<CertificateEncoding> for CertificateEncodingProto {
    fn from(value: CertificateEncoding) -> Self {
        match value {
            CertificateEncoding::Pem => CertificateEncodingProto::Pem,
        }
    }
}

#[derive(Clone)]
pub struct ClientCertificate {
    pub client_certificate: Vec<u8>,
    pub certificate_encoding: CertificateEncoding,
}

impl std::fmt::Debug for ClientCertificate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientCertificate")
            .field("client_certificate.len", &self.client_certificate)
            .field("certificate_encoding", &self.certificate_encoding)
            .finish()
    }
}

impl TryFrom<ClientCertificateProto> for ClientCertificate {
    type Error = PluginBootstrapDeserializationError;

    fn try_from(value: ClientCertificateProto) -> Result<Self, Self::Error> {
        if value.client_certificate.is_empty() {
            return Err(PluginBootstrapDeserializationError::EmptyField(
                "ClientCertificateProto.client_certificate",
            ));
        }

        let certificate_encoding = CertificateEncoding::try_from(value.certificate_encoding())?;

        Ok(Self {
            client_certificate: value.client_certificate,
            certificate_encoding,
        })
    }
}

impl From<ClientCertificate> for ClientCertificateProto {
    fn from(value: ClientCertificate) -> Self {
        let certificate_encoding: CertificateEncodingProto = value.certificate_encoding.into();
        Self {
            client_certificate: value.client_certificate,
            certificate_encoding: certificate_encoding as i32,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GetBootstrapInfoResponse {
    pub plugin_payload: PluginPayload,
    pub client_certificate: ClientCertificate,
}

impl TryFrom<GetBootstrapInfoResponseProto> for GetBootstrapInfoResponse {
    type Error = PluginBootstrapDeserializationError;

    fn try_from(value: GetBootstrapInfoResponseProto) -> Result<Self, Self::Error> {
        let plugin_payload = value
            .plugin_payload
            .ok_or(PluginBootstrapDeserializationError::MissingRequiredField(
                "GetBootstrapInfoResponseProto.plugin_payload",
            ))?
            .try_into()?;
        let client_certificate = value
            .client_certificate
            .ok_or(PluginBootstrapDeserializationError::MissingRequiredField(
                "GetBootstrapInfoResponseProto.client_certificate",
            ))?
            .try_into()?;
        Ok(GetBootstrapInfoResponse {
            plugin_payload,
            client_certificate,
        })
    }
}

impl From<GetBootstrapInfoResponse> for GetBootstrapInfoResponseProto {
    fn from(value: GetBootstrapInfoResponse) -> Self {
        Self {
            plugin_payload: Some(value.plugin_payload.into()),
            client_certificate: Some(value.client_certificate.into()),
        }
    }
}
