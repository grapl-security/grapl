use std::{
    io::Read,
    sync::atomic::Ordering,
};

use rust_proto::{
    graplinc::grapl::api::plugin_bootstrap::v1beta1::{
        server::PluginBootstrapApi,
        ClientCertificate,
        GetBootstrapRequest,
        GetBootstrapResponse,
        PluginPayload,
    },
    protocol::{
        error::ServeError,
        status::Status,
    },
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginBootstrapError {
    #[error("IoError {0}")]
    IoError(#[from] std::io::Error),
    #[error("ServeError {0}")]
    ServeError(#[from] ServeError),
}

impl From<PluginBootstrapError> for Status {
    fn from(e: PluginBootstrapError) -> Self {
        match e {
            PluginBootstrapError::IoError(e) => Status::unknown(e.to_string()),
            PluginBootstrapError::ServeError(e) => Status::internal(e.to_string()),
        }
    }
}

pub struct PluginBootstrapper {
    pub client_certificate: ClientCertificate,
    pub plugin_payload: PluginPayload,
    pub counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl PluginBootstrapper {
    pub fn new(client_certificate: ClientCertificate, plugin_payload: PluginPayload) -> Self {
        Self {
            client_certificate,
            plugin_payload,
            counter: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    pub fn load(
        certificate_path: &std::path::Path,
        plugin_binary_path: &std::path::Path,
    ) -> Result<Self, PluginBootstrapError> {
        let certificate_file = std::fs::File::open(certificate_path)?;
        let plugin_binary_file = std::fs::File::open(plugin_binary_path)?;

        let mut certificate = Vec::with_capacity(512);
        let mut plugin_binary = Vec::with_capacity(128_000_000);

        let mut reader = std::io::BufReader::new(certificate_file);
        reader.read_to_end(&mut certificate)?;

        let mut reader = std::io::BufReader::new(plugin_binary_file);
        reader.read_to_end(&mut plugin_binary)?;

        let plugin_payload = PluginPayload {
            plugin_binary: plugin_binary.into(),
        };

        let client_certificate = ClientCertificate {
            client_certificate: certificate.into(),
        };

        Ok(PluginBootstrapper::new(client_certificate, plugin_payload))
    }

    async fn get_bootstrap(&self) -> GetBootstrapResponse {
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);

        if counter != 0 {
            tracing::warn!(
                message="Bootstrap information has been requested more than once.",
                count=%counter,
            );
        }

        GetBootstrapResponse {
            plugin_payload: self.plugin_payload.clone(),
            client_certificate: self.client_certificate.clone(),
        }
    }
}

pub struct PluginBootstrap {
    plugin_bootstrapper: PluginBootstrapper,
}

impl PluginBootstrap {
    pub fn new(plugin_bootstrapper: PluginBootstrapper) -> PluginBootstrap {
        PluginBootstrap {
            plugin_bootstrapper,
        }
    }
}

#[async_trait::async_trait]
impl PluginBootstrapApi for PluginBootstrap {
    type Error = PluginBootstrapError;

    async fn get_bootstrap(
        &self,
        _request: GetBootstrapRequest,
    ) -> Result<GetBootstrapResponse, Self::Error> {
        Ok(self.plugin_bootstrapper.get_bootstrap().await)
    }
}
