#![cfg(feature = "integration")]

use std::collections::HashMap;

use async_trait::async_trait;
use nomad_client_gen::{
    apis::namespaces_api,
    models,
};
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
async fn test_create_namespace() -> Result<(), Box<dyn std::error::Error>> {
    let client = NomadClient::from_env();
    client
        .create_update_namespace(models::Namespace {
            name: Some("test-create-namespace".into()),
            description: Some("im a namespace".into()),
            ..Default::default()
        })
        .await?;
    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_create_namespace_2x_causes_update() -> Result<(), Box<dyn std::error::Error>> {
    let client = NomadClient::from_env();
    let name = "test-create-namespace-2x";
    let description1 = "im a namespace";
    let description2 = "im updated";
    client
        .create_update_namespace(models::Namespace {
            name: Some(name.into()),
            description: Some(description1.into()),
            ..Default::default()
        })
        .await?;
    let ns = client.get_namespace(name).await?;
    assert_eq!(ns.description.unwrap(), description1);

    client
        .create_update_namespace(models::Namespace {
            name: Some(name.into()),
            description: Some(description2.into()),
            ..Default::default()
        })
        .await?;
    let ns = client.get_namespace(name).await?;
    assert_eq!(ns.description.unwrap(), description2);
    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_plan_job_with_too_much_memory() -> Result<(), Box<dyn std::error::Error>> {
    let client = NomadClient::from_env();
    let job_hcl = TOO_MUCH_MEMORY_NOMAD_JOB;
    let job = NomadCli::default().parse_hcl2(job_hcl, HashMap::default())?;
    let plan_result = client.plan_job(&job, "too-much-memory-job", None).await?;
    match plan_result.ensure_allocation() {
        Err(NomadClientError::PlanJobAllocationFail) => Ok(()),
        _ => Err("Expected failed allocation".into()),
    }
}

#[async_trait]
trait TestFunctions {
    async fn get_namespace(
        &self,
        namespace_name: &str,
    ) -> Result<models::Namespace, Box<dyn std::error::Error>>;
}

#[async_trait]
impl TestFunctions for NomadClient {
    async fn get_namespace(
        &self,
        namespace_name: &str,
    ) -> Result<models::Namespace, Box<dyn std::error::Error>> {
        Ok(namespaces_api::get_namespace(
            &self.internal_config,
            namespaces_api::GetNamespaceParams {
                namespace_name: namespace_name.into(),
                ..Default::default()
            },
        )
        .await?)
    }
}
