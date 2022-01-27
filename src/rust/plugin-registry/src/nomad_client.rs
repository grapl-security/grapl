use std::net::SocketAddr;

use nomad_client_gen::{
    apis::{
        configuration::Configuration as InternalConfig,
        jobs_api,
        namespaces_api,
        Error,
    },
    models,
};
use structopt::StructOpt;

use crate::nomad_cli;

/// Represents the environment variables needed to construct a NomadClient
#[derive(StructOpt, Debug)]
pub struct NomadClientConfig {
    #[structopt(env)]
    /// "${attr.unique.network.ip-address}:4646
    nomad_service_address: SocketAddr,
}

/// A thin wrapper around the nomad_client_gen with usability improvements.
pub struct NomadClient {
    pub internal_config: InternalConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum NomadClientError {
    // Quick note: the error enums in the generated client *are not* std::error::Error
    #[error("ParseHclError {0:?}")]
    ParseHclError(#[from] nomad_cli::ParseHclError),
    #[error("CreateNamespaceError {0:?}")]
    CreateNamespaceErrror(#[from] Error<namespaces_api::PostNamespaceError>),
    #[error("CreateJobError {0:?}")]
    CreateJobError(#[from] Error<jobs_api::PostJobError>),
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

    pub async fn create_namespace(&self, name: &str) -> Result<(), NomadClientError> {
        let new_namespace = models::Namespace {
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
        .map_err(NomadClientError::from)
    }

    pub async fn create_job(
        &self,
        job: models::Job,
        namespace: Option<String>,
    ) -> Result<models::JobRegisterResponse, NomadClientError> {
        jobs_api::post_job(
            &self.internal_config,
            jobs_api::PostJobParams {
                namespace: namespace.clone(),
                job_name: "grapl-plugin".to_owned(),
                job_register_request: models::JobRegisterRequest {
                    namespace: namespace.clone(),
                    job: Some(job.into()),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await
        .map_err(NomadClientError::from)
    }
}
