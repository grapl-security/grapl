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
    #[error("CreateNamespaceError {0:?}")]
    CreateNamespaceErrror(#[from] Error<namespaces_api::PostNamespaceError>),
    #[error("CreateJobError {0:?}")]
    CreateJobError(#[from] Error<jobs_api::PostJobError>),
    #[error("PlanJobError {0:?}")]
    PlanJobError(#[from] Error<jobs_api::PostJobPlanError>),
    #[error("PlanJobAllocationFail")]
    PlanJobAllocationFail,
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

    /// Create or update a namespace (primary key'd on `name`)
    pub async fn create_update_namespace(&self, new_namespace: models::Namespace) -> Result<(), NomadClientError> {
        namespaces_api::post_namespace(
            // Shockingly, not `create_namespace()`
            &self.internal_config,
            namespaces_api::PostNamespaceParams {
                namespace2: new_namespace,
                ..Default::default()
            },
        )
        .await
        .map_err(NomadClientError::from)
    }

    pub async fn create_job(
        &self,
        job: &models::Job,
        job_name: &str,
        namespace: Option<String>,
    ) -> Result<models::JobRegisterResponse, NomadClientError> {
        jobs_api::post_job(
            &self.internal_config,
            jobs_api::PostJobParams {
                namespace: namespace.clone(),
                job_name: job_name.to_owned(),
                job_register_request: models::JobRegisterRequest {
                    job: Some(job.clone().into()),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await
        .map_err(NomadClientError::from)
    }

    pub async fn plan_job(
        &self,
        job: &models::Job,
        job_name: &str,
        namespace: Option<String>,
    ) -> Result<models::JobPlanResponse, NomadClientError> {
        jobs_api::post_job_plan(
            &self.internal_config,
            jobs_api::PostJobPlanParams {
                namespace: namespace.clone(),
                job_name: job_name.to_owned(),
                job_plan_request: models::JobPlanRequest {
                    job: Some(job.clone().into()),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await
        .map_err(NomadClientError::from)
    }
}

pub trait CanEnsureAllocation {
    fn ensure_allocation(&self) -> Result<(), NomadClientError>;
}

impl CanEnsureAllocation for models::JobPlanResponse {
    fn ensure_allocation(&self) -> Result<(), NomadClientError> {
        if let Some(failed_allocs) = &self.failed_tg_allocs {
            if !failed_allocs.is_empty() {
                tracing::warn!(message="Job failed to allocate", failed_allocs=?failed_allocs);
                return Err(NomadClientError::PlanJobAllocationFail);
            }
        }
        Ok(())
    }
}
