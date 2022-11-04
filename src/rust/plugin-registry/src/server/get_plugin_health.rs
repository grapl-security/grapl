use clap::Parser;
use consul_client::{
    ConsulClient,
    ConsulClientConfig,
};
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::PluginHealthStatus;

use super::plugin_nomad_job;
use crate::{
    db::{
        client::PluginRegistryDbClient,
        models::PluginDeploymentStatus,
    },
    error::PluginRegistryServiceError,
    nomad::client::NomadClient,
};

pub async fn get_plugin_health(
    nomad_client: &NomadClient,
    db_client: &PluginRegistryDbClient,
    plugin_id: uuid::Uuid,
) -> Result<PluginHealthStatus, PluginRegistryServiceError> {
    let plugin_deployment = db_client.get_plugin_deployment(&plugin_id).await;
    match plugin_deployment {
        Err(_) => Ok(PluginHealthStatus::NotDeployed),
        Ok(deploy) => {
            match deploy.status {
                PluginDeploymentStatus::Fail => {
                    // Perhaps this should be a different Status?
                    Ok(PluginHealthStatus::Dead)
                }
                PluginDeploymentStatus::Success => {
                    query_nomad_for_health(nomad_client, plugin_id).await
                }
            }
        }
    }
}

async fn query_nomad_for_health(
    nomad_client: &NomadClient,
    plugin_id: uuid::Uuid,
) -> Result<PluginHealthStatus, PluginRegistryServiceError> {
    let job_name = plugin_nomad_job::job_name().to_owned();
    let namespace_name = plugin_nomad_job::namespace_name(&plugin_id);
    let job = nomad_client.get_job(job_name, Some(namespace_name)).await?;
    match job.status {
        Some(status) => match status.as_str() {
            "pending" => Ok(PluginHealthStatus::Pending),
            // Okay, so now we know the job is running in Nomad; next let's
            // query Consul if the plugin service is healthy.
            "running" => query_consul_for_service_health(plugin_id).await,
            "dead" => Ok(PluginHealthStatus::Dead),
            other => Err(PluginRegistryServiceError::DeploymentStateError(format!(
                "Unknown state {other}"
            ))),
        },
        _ => Err(PluginRegistryServiceError::DeploymentStateError(
            "No State for this job? Is this even possible?".to_owned(),
        )),
    }
}

/// Given a plugin-id, query Consul to see if the service
/// `plugin-${plugin_id}` is healthy.
async fn query_consul_for_service_health(
    plugin_id: uuid::Uuid,
) -> Result<PluginHealthStatus, PluginRegistryServiceError> {
    let service_name = format!("plugin-{plugin_id}");
    let consul_client = ConsulClient::new(ConsulClientConfig::parse());
    let service_health = consul_client.check_health(service_name).await?;
    tracing::info!(message = "health result", service_health =? service_health);
    Ok(service_health.into())
}
