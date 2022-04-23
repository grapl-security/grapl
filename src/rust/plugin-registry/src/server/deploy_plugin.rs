use std::collections::HashMap;

use nomad_client_gen::models;

use super::{
    s3_url::get_s3_url,
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
    service_config: &PluginRegistryServiceConfig,
    cli: &NomadCli,
    plugin_runtime: &PluginRuntime,
) -> Result<models::Job, NomadCliError> {
    let plugin_artifact_url = {
        let key = &plugin.artifact_s3_key;
        let bucket = &service_config.plugin_s3_bucket_name;
        get_s3_url(bucket, key)
    };
    match plugin_runtime {
        PluginRuntime::HaxDocker => {
            let job_file_hcl = static_files::HAX_DOCKER_PLUGIN_JOB;
            let job_file_vars: NomadVars = HashMap::from([
                (
                    "aws_account_id",
                    service_config.plugin_s3_bucket_aws_account_id.to_owned(),
                ),
                ("plugin_artifact_url", plugin_artifact_url),
                (
                    "plugin_runtime_image",
                    service_config.hax_docker_plugin_runtime_image.to_owned(),
                ),
                ("plugin_id", plugin.plugin_id.to_string()),
                ("tenant_id", plugin.tenant_id.to_string()),
            ]);
            cli.parse_hcl2(job_file_hcl, job_file_vars)
        }
        PluginRuntime::Firecracker => {
            // This is currently dead code until we revive our Firecracker
            // efforts.
            let job_file_hcl = static_files::PLUGIN_JOB;
            let job_file_vars: NomadVars = HashMap::from([
                (
                    "aws_account_id",
                    service_config.plugin_s3_bucket_aws_account_id.to_owned(),
                ),
                (
                    "kernel_artifact_url",
                    service_config.kernel_artifact_url.to_owned(),
                ),
                ("plugin_artifact_url", plugin_artifact_url),
                (
                    "plugin_bootstrap_container_image",
                    service_config.plugin_bootstrap_container_image.to_owned(),
                ),
                (
                    "plugin_execution_container_image",
                    service_config.plugin_execution_container_image.to_owned(),
                ),
                ("plugin_id", plugin.plugin_id.to_string()),
                (
                    "rootfs_artifact_url",
                    service_config.rootfs_artifact_url.to_owned(),
                ),
                ("tenant_id", plugin.tenant_id.to_string()),
            ]);
            cli.parse_hcl2(job_file_hcl, job_file_vars)
        }
    }
}

/// https://github.com/grapl-security/grapl-rfcs/blob/main/text/0000-plugins.md#deployplugin-details
#[tracing::instrument(skip(client, cli, db_client, plugin), err)]
pub async fn deploy_plugin(
    client: &NomadClient,
    cli: &NomadCli,
    db_client: &PluginRegistryDbClient,
    plugin: PluginRow,
    service_config: &PluginRegistryServiceConfig,
) -> Result<(), PluginRegistryServiceError> {
    // --- Convert HCL to JSON Job model
    let job_name = "grapl-plugin"; // Matches what's in `plugin.nomad`

    let job = get_job(&plugin, service_config, cli, &HARDCODED_PLUGIN_RUNTIME)?;

    // --- Deploy namespace
    let namespace_name = format!("plugin-{id}", id = plugin.plugin_id);
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
        .map_err(|_| PluginRegistryServiceError::NomadJobAllocationError)?;

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

#[cfg(feature = "integration")]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn arbitrary_service_config() -> PluginRegistryServiceConfig {
        PluginRegistryServiceConfig {
            plugin_registry_bind_address: "1.2.3.4:1234".parse().unwrap(),
            hax_docker_plugin_runtime_image: Default::default(),
            kernel_artifact_url: Default::default(),
            plugin_bootstrap_container_image: Default::default(),
            plugin_execution_container_image: Default::default(),
            plugin_s3_bucket_aws_account_id: Default::default(),
            plugin_s3_bucket_name: Default::default(),
            rootfs_artifact_url: Default::default(),
        }
    }
    /// This is used to keep test coverage on the eventually-desirable-but-
    /// currently-deadcode `get_job` logic branch.
    /// NOTE:
    /// It doesn't really need to be an integration test, but as it relies on
    /// the `nomad` binary in the integration-test container, it's a decent
    /// excuse.
    #[test]
    fn test_get_job_firecracker() -> Result<(), Box<dyn std::error::Error>> {
        let arbitrary_uuid = uuid::Uuid::new_v4();
        let plugin = PluginRow {
            plugin_id: arbitrary_uuid,
            tenant_id: arbitrary_uuid,
            display_name: "arbitrary".to_owned(),
            plugin_type: "analyzer".to_owned(),
            artifact_s3_key: "arbitrary".to_owned(),
        };
        let service_config = arbitrary_service_config();
        let cli = NomadCli::default();
        let plugin_runtime = PluginRuntime::Firecracker;
        get_job(&plugin, &service_config, &cli, &plugin_runtime)?;
        Ok(())
    }
}
