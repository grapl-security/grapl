use std::collections::HashMap;

use nomad_client_gen::models;

use super::service::PluginRegistryServiceConfig;
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
            NomadVars,
        },
        client::{
            CanEnsureAllocation,
            NomadClient,
        },
    },
    static_files,
};

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
    let job = {
        let job_file_hcl = static_files::PLUGIN_JOB;
        // TODO: Deprecate this in next PR and replace with passed-in env variable
        // ~ wimax Feb 2022
        let kernel_artifact_url = format!(
            "https://{bucket}.s3.amazonaws.com/{key}",
            bucket = service_config.plugin_s3_bucket_name,
            key = "kernel/v0.tar.gz",
        );
        let rootfs_artifact_url = format!(
            "https://{bucket}.s3.amazonaws.com/{key}",
            bucket = service_config.plugin_s3_bucket_name,
            key = "rootfs/v0.rootfs.tar.gz",
        );
        let job_file_vars: NomadVars = HashMap::from([
            (
                "aws_account_id",
                service_config.plugin_s3_bucket_aws_account_id.to_string(),
            ),
            ("kernel_artifact_url", kernel_artifact_url),
            ("plugin_artifact_url", plugin.artifact_s3_key),
            (
                "plugin_bootstrap_container_image",
                service_config.plugin_bootstrap_container_image.to_owned(),
            ),
            (
                "plugin_execution_container_image",
                service_config.plugin_execution_container_image.to_owned(),
            ),
            ("plugin_id", plugin.plugin_id.to_string()),
            ("rootfs_artifact_url", rootfs_artifact_url),
            ("tenant_id", plugin.tenant_id.to_string()),
        ]);
        cli.parse_hcl2(job_file_hcl, job_file_vars)?
    };

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
