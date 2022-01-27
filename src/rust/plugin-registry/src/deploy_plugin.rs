use std::collections::HashMap;

use crate::{
    db_client::GetPluginRow,
    error::PluginRegistryServiceError,
    nomad_cli,
    nomad_client,
    static_files,
};

/// https://github.com/grapl-security/grapl-rfcs/blob/main/text/0000-plugins.md#deployplugin-details
#[tracing::instrument(skip(client, cli, plugin), err)]
pub async fn deploy_plugin(
    client: nomad_client::NomadClient,
    cli: nomad_cli::NomadCli,
    plugin: GetPluginRow,
    plugin_bucket_owner_id: &str,
) -> Result<(), PluginRegistryServiceError> {
    // --- Convert HCL to JSON Job model
    let job = {
        let job_file_hcl = static_files::PLUGIN_JOB;
        let job_file_vars: nomad_cli::NomadVars = HashMap::from([
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

    // TODO: "nomad job plan" to make sure we have enough memory/cpu for the task

    // --- Deploy namespace
    let namespace_name = &plugin.display_name; // TODO: Do we need to regex enforce display names?
    client.create_namespace(namespace_name).await?;
    // TODO: What if the namespace already exists?

    // --- Start the job
    let _job = client
        .create_job(job, Some(namespace_name.to_owned()))
        .await?;
    // There's no guarantee that the job is *healthy*, just that it's been deployed.

    // --- If success, mark plugin as being deployed in `plugins` table

    // TODO next CR. Right now all the plugins table interop is in the main
    //      server controller, gross!
    Ok(())
}
