/// Wrapper around Tonic's Identity so we can avoid clients having to depend
/// on Tonic directly.
pub struct Identity {
    inner: tonic::transport::Identity,
}

impl Identity {
    pub fn from_pem(cert: Vec<u8>, key: Vec<u8>) -> Self {
        Self {
            inner: tonic::transport::Identity::from_pem(cert, key),
        }
    }
}

impl From<Identity> for tonic::transport::Identity {
    fn from(identity: Identity) -> tonic::transport::Identity {
        identity.inner
    }
}

#[derive(Clone)]
pub struct Certificate {
    inner: tonic::transport::Certificate,
}
impl Certificate {
    pub fn from_pem(pem: Vec<u8>) -> Self {
        Self {
            inner: tonic::transport::Certificate::from_pem(pem),
        }
    }
}
impl From<Certificate> for tonic::transport::Certificate {
    fn from(certificate: Certificate) -> tonic::transport::Certificate {
        certificate.inner
    }
}

pub struct ClientTlsConfig {
    inner: tonic::transport::ClientTlsConfig,
}
impl ClientTlsConfig {
    pub fn new(ca_certificate: Certificate, domain_name: &str) -> Self {
        Self {
            inner: tonic::transport::ClientTlsConfig::new()
                .ca_certificate(ca_certificate.into())
                .domain_name(domain_name),
        }
    }
}
impl From<ClientTlsConfig> for tonic::transport::ClientTlsConfig {
    fn from(config: ClientTlsConfig) -> tonic::transport::ClientTlsConfig {
        config.inner
    }
}
