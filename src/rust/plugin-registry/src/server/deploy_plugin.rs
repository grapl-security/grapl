use std::collections::HashMap;

use nomad_client_gen::models;

use crate::{
    db::client::PluginRow,
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
#[tracing::instrument(skip(client, cli, plugin), err)]
pub async fn deploy_plugin(
    client: &NomadClient,
    cli: &NomadCli,
    plugin: PluginRow,
    plugin_bucket_owner_id: &str,
) -> Result<(), PluginRegistryServiceError> {
    // --- Convert HCL to JSON Job model
    let job_name = "grapl-plugin"; // Matches what's in `plugin.nomad`
    let job = {
        let job_file_hcl = static_files::PLUGIN_JOB;
        let job_file_vars: NomadVars = HashMap::from([
            ("plugin_id".to_owned(), plugin.plugin_id.to_string()),
            ("tenant_id".to_owned(), plugin.tenant_id.to_string()),
            ("plugin_artifact_url".to_owned(), plugin.artifact_s3_key),
            (
                "aws_account_id".to_owned(),
                plugin_bucket_owner_id.to_string(),
            ),
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
    let _job = client
        .create_job(&job, job_name, Some(namespace_name.clone()))
        .await?;
    // There's no guarantee that the job is *healthy*, just that it's been deployed.

    // --- If success, mark plugin as being deployed in `plugins` table

    // TODO next CR. Right now all the plugins table interop is in the main
    //      server controller, gross!
    Ok(())
}
