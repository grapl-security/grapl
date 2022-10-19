use std::collections::HashMap;

use nomad_client_gen::models;
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::PluginType;

use super::{
    plugin_nomad_job,
    s3_uri::get_s3_uri,
    service::PluginRegistryServiceConfig,
};
use crate::{
    db::{
        client::PluginRegistryDbClient,
        models::{
            PluginDeploymentStatus,
            PluginRow,
        },
    },
    error::PluginRegistryServiceError,
    nomad::{
        cli::{
            NomadCli,
            NomadCliError,
            NomadVars,
        },
        client::{
            CanEnsureAllocation,
            NomadClient,
        },
    },
    static_files,
};

/**
 * https://github.com/grapl-security/issue-tracker/issues/908
 * This enum can eventually be removed once we remove HaxDocker.
 * I am primarily using it to keep test coverage on the otherwise-deadcode
 * PluginRuntime::Firecracker stuff.
 */

#[allow(dead_code)]
pub enum PluginRuntime {
    HaxDocker,
    Firecracker,
}
static HARDCODED_PLUGIN_RUNTIME: PluginRuntime = PluginRuntime::HaxDocker;

pub fn get_job(
    plugin: &PluginRow,
    service_config: PluginRegistryServiceConfig,
    cli: &NomadCli,
    plugin_runtime: &PluginRuntime,
) -> Result<models::Job, NomadCliError> {
    let plugin_artifact_url = {
        let key = &plugin.artifact_s3_key;
        let bucket = &service_config.bucket_name;
        get_s3_uri(bucket, key)
    };
    let passthru = service_config.passthrough_vars;
    let plugin_type = PluginType::try_from(plugin.plugin_type.as_str())
        .expect("Unknown plugin-type in DB is bad news");
    let plugin_execution_sidecar_image = match plugin_type {
        PluginType::Generator => passthru.generator_sidecar_image,
        PluginType::Analyzer => passthru.analyzer_sidecar_image,
    };
    match plugin_runtime {
        PluginRuntime::HaxDocker => {
            let hax_docker_nomad_job = match plugin_type {
                PluginType::Generator => static_files::HAX_DOCKER_GENERATOR_JOB,
                PluginType::Analyzer => static_files::HAX_DOCKER_ANALYZER_JOB,
            };
            let mut job_file_vars: NomadVars = HashMap::from([
                ("aws_account_id", service_config.bucket_aws_account_id),
                ("plugin_artifact_url", plugin_artifact_url),
                (
                    "plugin_runtime_image",
                    service_config.hax_docker_plugin_runtime_image,
                ),
                (
                    "plugin_execution_sidecar_image",
                    plugin_execution_sidecar_image,
                ),
                ("plugin_id", plugin.plugin_id.to_string()),
                ("tenant_id", plugin.tenant_id.to_string()),
                // Passthrough vars
                ("rust_log", passthru.rust_log),
            ]);
            if plugin_type == PluginType::Analyzer {
                job_file_vars.insert("graph_query_proxy_image", passthru.graph_query_proxy_image);
            }
            cli.parse_hcl2(hax_docker_nomad_job, job_file_vars)
        }
        PluginRuntime::Firecracker => {
            // This is currently dead code until we revive our Firecracker
            // efforts.
            let job_file_hcl = static_files::PLUGIN_JOB;
            let job_file_vars: NomadVars = HashMap::from([
                ("aws_account_id", service_config.bucket_aws_account_id),
                ("kernel_artifact_url", service_config.kernel_artifact_url),
                ("plugin_artifact_url", plugin_artifact_url),
                (
                    "plugin_bootstrap_container_image",
                    service_config.plugin_bootstrap_container_image,
                ),
                (
                    "plugin_execution_sidecar_image",
                    plugin_execution_sidecar_image,
                ),
                ("plugin_id", plugin.plugin_id.to_string()),
                ("rootfs_artifact_url", service_config.rootfs_artifact_url),
                ("tenant_id", plugin.tenant_id.to_string()),
            ]);
            cli.parse_hcl2(job_file_hcl, job_file_vars)
        }
    }
}

/// https://github.com/grapl-security/grapl-rfcs/blob/main/text/0000-plugins.md#deployplugin-details
#[tracing::instrument(skip(client, cli, db_client, plugin, service_config), err)]
pub async fn deploy_plugin(
    client: &NomadClient,
    cli: &NomadCli,
    db_client: &PluginRegistryDbClient,
    plugin: PluginRow,
    service_config: &PluginRegistryServiceConfig,
) -> Result<(), PluginRegistryServiceError> {
    // --- Convert HCL to JSON Job model
    let job_name = plugin_nomad_job::job_name();

    let job = get_job(
        &plugin,
        service_config.clone(),
        cli,
        &HARDCODED_PLUGIN_RUNTIME,
    )?;

    // --- Deploy namespace
    let namespace_name = plugin_nomad_job::namespace_name(&plugin.plugin_id);
    let namespace_description = format!("Plugin for {name}", name = plugin.display_name);
    client
        .create_update_namespace(models::Namespace {
            name: namespace_name.clone().into(),
            description: namespace_description.into(),
            ..Default::default()
        })
        .await?;

    // --- Make sure that the Nomad agents can accept the job
    let plan_result = client
        .plan_job(&job, job_name, namespace_name.clone().into())
        .await?;
    plan_result
        .ensure_allocation()
        .map_err(|e| PluginRegistryServiceError::NomadJobAllocationError(e))?;

    // --- Start the job
    let job_result = client
        .create_job(&job, job_name, Some(namespace_name.clone()))
        .await;
    // There's no guarantee that the job is *healthy*, just that it's been deployed.

    // --- If success, mark plugin as being deployed in `plugins` table
    let status = PluginDeploymentStatus::from(&job_result);
    db_client
        .create_plugin_deployment(&plugin.plugin_id, status)
        .await?;

    job_result?;

    // TODO next CR. Right now all the plugins table interop is in the main
    //      server controller, gross!
    Ok(())
}

#[tracing::instrument(skip(client, db_client, plugin), err)]
pub async fn teardown_plugin(
    client: &NomadClient,
    db_client: &PluginRegistryDbClient,
    plugin: PluginRow,
    service_config: &PluginRegistryServiceConfig,
) -> Result<(), PluginRegistryServiceError> {
    // --- Convert HCL to JSON Job model
    let job_name = plugin_nomad_job::job_name();
    let namespace_name = plugin_nomad_job::namespace_name(&plugin.plugin_id);

    // --- Delete the job
    client
        .delete_job(job_name.to_owned(), Some(namespace_name.clone()))
        .await?;

    // --- Mark plugin as inactive in `plugins` table
    db_client
        .deactivate_plugin_deployment(&plugin.plugin_id)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arbitrary_service_config() -> PluginRegistryServiceConfig {
        PluginRegistryServiceConfig {
            plugin_registry_bind_address: "1.2.3.4:1234".parse().unwrap(),
            hax_docker_plugin_runtime_image: Default::default(),
            kernel_artifact_url: Default::default(),
            plugin_bootstrap_container_image: Default::default(),
            bucket_aws_account_id: Default::default(),
            bucket_name: Default::default(),
            rootfs_artifact_url: Default::default(),
            artifact_size_limit_mb: Default::default(),
            passthrough_vars: Default::default(),
        }
    }

    /// This is used to keep test coverage on the eventually-desirable-but-
    /// currently-deadcode `get_job` logic branch.
    #[test]
    fn test_get_job_firecracker() -> Result<(), Box<dyn std::error::Error>> {
        let arbitrary_uuid = uuid::Uuid::new_v4();
        let plugin = PluginRow {
            plugin_id: arbitrary_uuid,
            tenant_id: arbitrary_uuid,
            display_name: "arbitrary".to_owned(),
            plugin_type: "generator".to_owned(),
            event_source_id: None,
            artifact_s3_key: "arbitrary".to_owned(),
        };
        let service_config = arbitrary_service_config();
        let cli = NomadCli::default();
        let plugin_runtime = PluginRuntime::Firecracker;
        get_job(&plugin, service_config, &cli, &plugin_runtime)?;
        Ok(())
    }
}
