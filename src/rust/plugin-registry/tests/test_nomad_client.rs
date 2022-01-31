#![cfg(feature = "integration")]

use std::collections::HashMap;

use nomad_client_gen::models;
use plugin_registry::nomad::{
    cli::NomadCli,
    client::{
        CanEnsureAllocation,
        NomadClient,
        NomadClientError,
    },
};

pub static TOO_MUCH_MEMORY_NOMAD_JOB: &'static str = include_str!("too_much_memory.nomad");

#[test_log::test(tokio::test)]
async fn test_nomad_client_create_namespace() -> Result<(), Box<dyn std::error::Error>> {
    let client = NomadClient::from_env();
    client
        .create_namespace(models::Namespace{
            name: "test-nomad-client-create-namespace",
            description: "im a namespace",
            ..Default::default()
        })
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
    match plan_result.ensure_allocation() {
        Err(NomadClientError::PlanJobAllocationFail) => Ok(()),
        _ => Err("Expected failed allocation".into()),
    }
}
