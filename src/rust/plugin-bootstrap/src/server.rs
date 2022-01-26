use std::{
    io::Read,
    sync::atomic::Ordering,
};

use rust_proto::plugin_bootstrap::{
    ClientCertificate,
    GetBootstrapInfoRequestProto,
    GetBootstrapInfoResponse,
    GetBootstrapInfoResponseProto,
    PluginBootstrapService,
    PluginBootstrapServiceServer,
    PluginPayload,
};
use tonic::{
    transport::Server,
    Status,
};

use crate::PluginBootstrapServiceConfig;

#[derive(Debug, thiserror::Error)]
pub enum PluginBootstrapperError {
    #[error("IoError {0}")]
    IoError(#[from] std::io::Error),
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
    ) -> Result<Self, PluginBootstrapperError> {
        let certificate_file = std::fs::File::open(certificate_path)?;
        let plugin_binary_file = std::fs::File::open(plugin_binary_path)?;

        let mut certificate = Vec::with_capacity(512);
        let mut plugin_binary = Vec::with_capacity(128_000_000);

        let mut reader = std::io::BufReader::new(certificate_file);
        reader.read_to_end(&mut certificate)?;
        let mut reader = std::io::BufReader::new(plugin_binary_file);
        reader.read_to_end(&mut plugin_binary)?;

        let plugin_payload = PluginPayload { plugin_binary };
        let client_certificate = ClientCertificate {
            client_certificate: certificate,
        };
        Ok(PluginBootstrapper::new(client_certificate, plugin_payload))
    }

    async fn get_bootstrap_info(&self) -> GetBootstrapInfoResponse {
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        if counter != 0 {
            tracing::warn!(
                message="Bootstrap information has been requested more than once.",
                count=%counter,
            );
        }
        GetBootstrapInfoResponse {
            plugin_payload: self.plugin_payload.clone(),
            client_certificate: self.client_certificate.clone(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn serve(
        self,
        service_config: PluginBootstrapServiceConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
        health_reporter
            .set_serving::<PluginBootstrapServiceServer<PluginBootstrapper>>()
            .await;

        let addr = service_config.plugin_registry_bind_address;
        tracing::info!(
            message="Starting PluginBootstrap",
            addr=?addr,
        );

        Server::builder()
            .trace_fn(|request| {
                tracing::info_span!(
                    "PluginBootstrap",
                    headers = ?request.headers(),
                    method = ?request.method(),
                    uri = %request.uri(),
                    extensions = ?request.extensions(),
                )
            })
            .add_service(health_service)
            .add_service(PluginBootstrapServiceServer::new(self))
            .serve(addr)
            .await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl PluginBootstrapService for PluginBootstrapper {
    #[tracing::instrument(skip(self))]
    async fn get_bootstrap_info(
        &self,
        _request: tonic::Request<GetBootstrapInfoRequestProto>,
    ) -> Result<tonic::Response<GetBootstrapInfoResponseProto>, Status> {
        let response = self.get_bootstrap_info().await;
        Ok(tonic::Response::new(response.into()))
    }
}
