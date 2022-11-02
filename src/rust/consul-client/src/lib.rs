use std::net::SocketAddr;

pub mod models;

#[derive(thiserror::Error, Debug)]
pub enum ConsulClientError {
    #[error("consul check health error: '{0}'")]
    CheckHealthError(reqwest::Error),
    #[error("reqwest serde error: '{0}'")]
    ReqwestSerdeError(reqwest::Error),
}

#[derive(clap::Parser, Debug)]
pub struct ConsulClientConfig {
    #[clap(long, env)]
    /// "${attr.unique.network.ip-address}:8500
    consul_service_address: SocketAddr,
}

/// A thin wrapper around the nomad_client_gen with usability improvements.
pub struct ConsulClient {
    address: String,
}
impl ConsulClient {
    pub fn new(config: ConsulClientConfig) -> Self {
        let address = config.consul_service_address.to_string();
        let address = format!("http://{address}");
        Self { address }
    }

    pub async fn check_health(
        &self,
        service_name: impl Into<String>,
    ) -> Result<models::CheckHealthResponse, ConsulClientError> {
        let service_name = service_name.into();
        let address = &self.address;
        let url = format!("{address}/v1/health/checks/{service_name}");
        let response = reqwest::get(url.clone())
            .await
            .map_err(ConsulClientError::CheckHealthError)?;
        let responses = response
            .json::<Vec<models::CheckHealthResponseElem>>()
            .await
            .map_err(ConsulClientError::ReqwestSerdeError)?;
        Ok(models::CheckHealthResponse(responses))
    }
}
