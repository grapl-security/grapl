use nomad_client_gen::{
    apis::{
        configuration::Configuration as InternalConfig,
        namespaces_api,
        Error,
    },
    models::Namespace,
};
use structopt::StructOpt;

/// Represents the environment variables needed to construct a NomadClient
#[derive(StructOpt, Debug)]
pub struct NomadClientConfig {
    #[structopt(env)]
    /// "${attr.unique.network.ip-address}:4646
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
            base_path: format!("http://{}/v1", nomad_client_config.nomad_service_address),
            ..Default::default()
        };

        NomadClient { internal_config }
    }

    pub async fn create_namespace(
        &self,
        name: &str,
    ) -> Result<(), Error<namespaces_api::PostNamespaceError>> {
        let new_namespace = Namespace {
            name: Some(name.to_owned()),
            description: Some("created by NomadClient::create_namespace".to_owned()),
            ..Default::default()
        };

        namespaces_api::post_namespace(
            // Shockingly, not `create_namespace()`
            &self.internal_config,
            namespaces_api::PostNamespaceParams {
                // It's odd to me that I have to specify the name twice...
                namespace_name: name.to_owned(),
                namespace2: new_namespace,
                ..Default::default()
            },
        )
        .await
    }
}
