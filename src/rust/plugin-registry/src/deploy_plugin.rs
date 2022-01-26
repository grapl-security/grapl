use std::collections::HashMap;

use crate::{
    nomad_cli,
    nomad_client,
    static_files,
};

/// https://github.com/grapl-security/grapl-rfcs/blob/main/text/0000-plugins.md#deployplugin-details
#[tracing::instrument(skip(client, plugin_id), err)]
pub async fn deploy_plugin(
    client: nomad_client::NomadClient,
    plugin_id: uuid::Uuid,
) -> Result<(), nomad_client::NomadClientError> {
    // Convert HCL to JSON Job model
    let job_file_hcl = static_files::PLUGIN_JOB;
    let job_file_vars: nomad_cli::NomadVars = HashMap::from([
        ("plugin_id".to_owned(), format!("{}", plugin_id)),
        ("tenant_id".to_owned(), "TODO".to_owned()),
        ("plugin_artifact_url".to_owned(), "TODO".to_owned()),
        ("aws_account_id".to_owned(), "TODO".to_owned()),
    ]);
    let job = nomad_cli::parse_hcl2(job_file_hcl, job_file_vars)
        .map_err(nomad_client::NomadClientError::from)?;

    // TODO: Should I "nomad job plan" to make sure we have enough memory/cpu for the task?

    // Deploy namespace
    let namespace_name = "todo-fill-this-in-with-real-plugin-name";
    client.create_namespace(namespace_name).await?;
    // TODO: What if the namespace already exists?

    // Start the job
    let _job = client
        .create_job(job, Some(namespace_name.to_owned()))
        .await?;
    // There's no guarantee that the job is *healthy*, just that it's been deployed.

    // If success, mark plugin as being deployed in `plugins` table

    // TODO next CR. Right now all the plugins table interop is in the main
    //      server controller, gross!
    Ok(())
}
