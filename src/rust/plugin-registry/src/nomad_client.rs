use nomad_client_gen::apis::configuration::Configuration as InternalConfig;
use nomad_client_gen::apis::namespaces_api;
use nomad_client_gen::apis::Error;
use structopt::StructOpt;

/// Represents the environment variables needed to construct a NomadClient
#[derive(StructOpt, Debug)]
pub struct NomadClientConfig {
    #[structopt(env)]
    nomad_service_address: String,
}

/// A thin wrapper around the nomad_client_gen
pub struct NomadClient {
    pub internal_config: InternalConfig,
}

#[allow(dead_code)]
impl NomadClient {
    /// Create a client from environment
    pub fn from_env() -> Self {
        Self::from_client_config(NomadClientConfig::from_args())
    }

    pub fn from_client_config(nomad_client_config: NomadClientConfig) -> Self {
        let internal_config = InternalConfig {
            base_path: format!("https://{}/v1", nomad_client_config.nomad_service_address),
            ..Default::default()
        };

        NomadClient { internal_config,}
    }

    pub async fn create_namespace(&self) -> Result<(), Error<namespaces_api::CreateNamespaceError>> {
        namespaces_api::create_namespace(&self.internal_config, None, Some("im a namespace"),
            None, None).await
    }
}
