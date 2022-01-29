use std::collections::HashMap;

use crate::{
    db::client::GetPluginRow,
    error::PluginRegistryServiceError,
    nomad::{
        cli::{
            NomadCli,
            NomadVars,
        },
        client::NomadClient,
    },
    static_files,
};

/// https://github.com/grapl-security/grapl-rfcs/blob/main/text/0000-plugins.md#deployplugin-details
#[tracing::instrument(skip(client, cli, plugin), err)]
pub async fn deploy_plugin(
    client: &NomadClient,
    cli: &NomadCli,
    plugin: GetPluginRow,
    plugin_bucket_owner_id: &str,
) -> Result<(), PluginRegistryServiceError> {
    // --- Convert HCL to JSON Job model
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
    let namespace_name = (&plugin.display_name).to_owned(); // TODO: Do we need to regex enforce display names?
    client.create_namespace(&namespace_name).await?;
    // TODO: What if the namespace already exists?

    // --- Make sure that the Nomad agents can accept the job
    let plan_result = client.plan_job(&job, Some(namespace_name.clone())).await?;
    if let Some(failed_allocs) = plan_result.failed_tg_allocs {
        if !failed_allocs.is_empty() {
            tracing::warn!(message="Job failed to allocate", failed_allocs=?failed_allocs);
            return Err(PluginRegistryServiceError::NomadJobAllocationError);
        }
    }

    // --- Start the job
    let _job = client
        .create_job(&job, Some(namespace_name.clone()))
        .await?;
    // There's no guarantee that the job is *healthy*, just that it's been deployed.

    // --- If success, mark plugin as being deployed in `plugins` table

    // TODO next CR. Right now all the plugins table interop is in the main
    //      server controller, gross!
    Ok(())
}
