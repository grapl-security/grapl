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
    let namespace_name = "cool-namespace";
    // 1. Deploy namespace
    client.create_namespace(namespace_name).await?;
    // TODO: What if the namespace already exists?

    // 2. Convert HCL to JSON
    let job_file_hcl = static_files::PLUGIN_JOB;
    let job_file_vars: nomad_cli::NomadVars = HashMap::from([
        ("plugin_id".to_owned(), format!("{}", plugin_id)),
        ("tenant_id".to_owned(), "TODO".to_owned()),
        ("plugin_artifact_url".to_owned(), "TODO".to_owned()),
        ("aws_account_id".to_owned(), "TODO".to_owned()),
    ]);
    let job = nomad_cli::parse_hcl2(job_file_hcl, job_file_vars)
        .map_err(nomad_client::NomadClientError::from)?;
    tracing::debug!(
        message = "The job is",
        job=?job,
    );

    // TODO: Should I "nomad job plan" to make sure we have enough memory/cpu?

    // 3. Start the job
    let _job = client
        .create_job(job, Some(namespace_name.to_owned()))
        .await?;

    // TODO: We want detach=False, so we should look at 'monitor':
    // https://github.com/hashicorp/nomad/blob/0af476263b5c6f12a43a069070400e1762ad17c2/command/job_run.go#L335

    // 4. If success, mark plugin as being deployed in `plugins` table
    Ok(())
}
