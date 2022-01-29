#![cfg(feature = "integration")]

use std::collections::HashMap;

use plugin_registry::nomad::{
    cli::NomadCli,
    client::NomadClient,
};

pub static TOO_MUCH_MEMORY_NOMAD_JOB: &'static str = include_str!("too_much_memory.nomad");

#[test_log::test(tokio::test)]
async fn test_nomad_client_create_namespace() -> Result<(), Box<dyn std::error::Error>> {
    let client = NomadClient::from_env();
    client
        .create_namespace("test-nomad-client-create-namespace")
        .await?;
    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_nomad_client_plan_job_with_too_much_memory() -> Result<(), Box<dyn std::error::Error>>
{
    let client = NomadClient::from_env();
    let job_hcl = TOO_MUCH_MEMORY_NOMAD_JOB;
    let job = NomadCli::default().parse_hcl2(job_hcl, HashMap::default())?;
    let plan_result = client.plan_job(&job, "too-much-memory-job", None).await?;
    if let Some(_) = plan_result.failed_tg_allocs {
        Ok(())
    } else {
        Err("Expected failed tg_allocs".into())
    }
}
