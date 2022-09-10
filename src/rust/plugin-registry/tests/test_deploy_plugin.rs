#![cfg(feature = "integration_tests")]

use std::time::Duration;

use bytes::Bytes;
use clap::Parser;
use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::graplinc::grapl::api::{
    client_factory::{
        build_grpc_client,
        services::PluginRegistryClientConfig,
    },
    plugin_registry::v1beta1::{
        DeployPluginRequest,
        GetPluginDeploymentRequest,
        GetPluginHealthRequest,
        GetPluginHealthResponse,
        PluginDeploymentStatus,
        PluginHealthStatus,
        PluginMetadata,
        PluginRegistryServiceClient,
        PluginType,
        TearDownPluginRequest,
    },
    protocol::{
        error::GrpcClientError,
        status::Code,
    },
};

pub const SMALL_TEST_BINARY: &'static [u8] = include_bytes!("./small_test_binary.sh");

fn get_example_generator() -> Result<Bytes, std::io::Error> {
    std::fs::read("/test-fixtures/example-generator").map(Bytes::from)
}

fn get_sysmon_generator() -> Result<Bytes, std::io::Error> {
    std::fs::read("/test-fixtures/sysmon-generator").map(Bytes::from)
}

#[test_log::test(tokio::test)]
async fn test_deploy_example_generator() -> eyre::Result<()> {
    let client_config = PluginRegistryClientConfig::parse();
    let mut client = build_grpc_client(client_config).await?;

    let tenant_id = uuid::Uuid::new_v4();
    let event_source_id = uuid::Uuid::new_v4();

    let create_response = {
        let display_name = uuid::Uuid::new_v4().to_string();
        let artifact = get_example_generator()?;
        let metadata = PluginMetadata::new(
            tenant_id,
            display_name.clone(),
            PluginType::Generator,
            Some(event_source_id.clone()),
        );

        client
            .create_plugin(
                metadata,
                futures::stream::once(async move { artifact.clone() }),
            )
            .timeout(std::time::Duration::from_secs(5))
            .await??
    };

    let plugin_id = create_response.plugin_id();

    let request = DeployPluginRequest::new(plugin_id);

    let _response = client
        .deploy_plugin(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    let plugin_deployment = client
        .get_plugin_deployment(GetPluginDeploymentRequest::new(plugin_id))
        .await?
        .plugin_deployment();

    assert_eq!(plugin_deployment.plugin_id(), plugin_id);
    assert!(plugin_deployment.deployed());
    assert_eq!(plugin_deployment.status(), PluginDeploymentStatus::Success);

    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_deploy_sysmon_generator() -> eyre::Result<()> {
    let client_config = PluginRegistryClientConfig::parse();
    let mut client = build_grpc_client(client_config).await?;

    let tenant_id = uuid::Uuid::new_v4();
    let event_source_id = uuid::Uuid::new_v4();

    let create_response = {
        let display_name = "sysmon-generator";
        let artifact = get_sysmon_generator()?;
        let metadata = PluginMetadata::new(
            tenant_id,
            display_name.to_owned(),
            PluginType::Generator,
            Some(event_source_id.clone()),
        );

        client
            .create_plugin(
                metadata,
                futures::stream::once(async move { artifact.clone() }),
            )
            .timeout(std::time::Duration::from_secs(5))
            .await??
    };

    let plugin_id = create_response.plugin_id();

    // Ensure that an un-deployed plugin is NotDeployed
    assert_health(&mut client, plugin_id, PluginHealthStatus::NotDeployed).await?;

    let _deploy_response = client
        .deploy_plugin(DeployPluginRequest::new(plugin_id))
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    // Let the task run for a bit. Tasks may potentially restart - e.g. if the
    // sidecar comes up before the main task, it'll panic because it expected a
    // healthy main-task health check.
    // Anyway: we let it run for a bit and _then_ check task health.
    tokio::time::sleep(Duration::from_secs(15)).await;

    // Ensure that a now-deployed plugin is now Running
    // If it's Pending, it's possible the agent is out of mem or disk
    // and was unable to allocate it.
    assert_health(&mut client, plugin_id, PluginHealthStatus::Running).await?;

    Ok(())
}

fn assert_contains(input: &str, expected_substr: &str) {
    assert!(
        input.contains(expected_substr),
        "Expected input '{input}' to contain '{expected_substr}'"
    )
}

async fn assert_health(
    client: &mut PluginRegistryServiceClient,
    plugin_id: uuid::Uuid,
    expected: PluginHealthStatus,
) -> eyre::Result<()> {
    let get_health_response: GetPluginHealthResponse = client
        .get_plugin_health(GetPluginHealthRequest::new(plugin_id))
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    let actual = get_health_response.health_status();
    if expected == actual {
        Ok(())
    } else {
        Err(eyre::eyre!("Expected one of {expected:?}, got {actual:?}"))
    }
}

#[test_log::test(tokio::test)]
/// So we *expect* this call to fail since it's an arbitrary PluginID that
/// hasn't been created yet
async fn test_deploy_plugin_but_plugin_id_doesnt_exist() -> eyre::Result<()> {
    let client_config = PluginRegistryClientConfig::parse();
    let mut client = build_grpc_client(client_config).await?;

    let randomly_selected_plugin_id = uuid::Uuid::new_v4();

    let request = DeployPluginRequest::new(randomly_selected_plugin_id);

    let response = client
        .deploy_plugin(request)
        .timeout(std::time::Duration::from_secs(5))
        .await?;

    match response {
        Err(GrpcClientError::ErrorStatus(s)) => {
            assert_eq!(s.code(), Code::NotFound);
            assert_contains(s.message(), &sqlx::Error::RowNotFound.to_string());
        }
        _ => panic!("Expected an error"),
    };
    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_teardown_plugin() {
    let client_config = PluginRegistryClientConfig::parse();
    let mut client = build_grpc_client(client_config)
        .await
        .expect("failed to build grpc client");

    let tenant_id = uuid::Uuid::new_v4();
    let event_source_id = uuid::Uuid::new_v4();

    let create_response = {
        let display_name = "sysmon-generator";
        let artifact = get_sysmon_generator().expect("failed to get sysmon generator");
        let metadata = PluginMetadata::new(
            tenant_id,
            display_name.to_owned(),
            PluginType::Generator,
            Some(event_source_id.clone()),
        );

        client
            .create_plugin(
                metadata,
                futures::stream::once(async move { artifact.clone() }),
            )
            .timeout(std::time::Duration::from_secs(5))
            .await
            .expect("timeout elapsed")
            .expect("failed to create plugin")
    };

    let plugin_id = create_response.plugin_id();

    // Ensure that an un-deployed plugin is NotDeployed
    assert_health(&mut client, plugin_id, PluginHealthStatus::NotDeployed)
        .await
        .expect("failed to assert health");

    client
        .deploy_plugin(DeployPluginRequest::new(plugin_id))
        .timeout(std::time::Duration::from_secs(5))
        .await
        .expect("timeout elapsed")
        .expect("failed to deploy plugin");

    client
        .tear_down_plugin(TearDownPluginRequest::new(plugin_id))
        .timeout(std::time::Duration::from_secs(5))
        .await
        .expect("timeout elapsed")
        .expect("failed to tear down plugin");

    let plugin_deployment = client
        .get_plugin_deployment(GetPluginDeploymentRequest::new(plugin_id))
        .await
        .expect("failed to get plugin deployment")
        .plugin_deployment();

    assert_eq!(plugin_deployment.plugin_id(), plugin_id);
    assert!(!plugin_deployment.deployed());
    assert_eq!(plugin_deployment.status(), PluginDeploymentStatus::Success);
}
