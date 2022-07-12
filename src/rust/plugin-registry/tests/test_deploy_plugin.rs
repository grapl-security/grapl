#![cfg(feature = "integration_tests")]

use bytes::Bytes;
use grapl_utils::future_ext::GraplFutureExt;
use plugin_registry::client::FromEnv;
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    DeployPluginRequest,
    PluginMetadata,
    PluginRegistryServiceClient,
    PluginRegistryServiceClientError,
    PluginType,
};

pub const SMALL_TEST_BINARY: &'static [u8] = include_bytes!("./small_test_binary.sh");

fn get_example_generator() -> Result<Bytes, std::io::Error> {
    std::fs::read("/test-fixtures/example-generator").map(Bytes::from)
}

fn get_sysmon_generator() -> Result<Bytes, std::io::Error> {
    std::fs::read("/test-fixtures/sysmon-generator").map(Bytes::from)
}

#[test_log::test(tokio::test)]
async fn test_deploy_example_generator() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PluginRegistryServiceClient::from_env().await?;

    let tenant_id = uuid::Uuid::new_v4();
    let event_source_id = uuid::Uuid::new_v4();

    let create_response = {
        let display_name = uuid::Uuid::new_v4().to_string();
        let artifact = get_example_generator()?;
        let metadata = PluginMetadata {
            tenant_id: tenant_id.clone(),
            display_name: display_name.clone(),
            plugin_type: PluginType::Generator,
            event_source_id: Some(event_source_id.clone()),
        };

        client
            .create_plugin(
                metadata,
                futures::stream::once(async move { artifact.clone() }),
            )
            .timeout(std::time::Duration::from_secs(5))
            .await??
    };

    let plugin_id = create_response.plugin_id;

    let request = DeployPluginRequest { plugin_id };

    let _response = client
        .deploy_plugin(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_deploy_sysmon_generator() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PluginRegistryServiceClient::from_env().await?;

    let tenant_id = uuid::Uuid::new_v4();
    let event_source_id = uuid::Uuid::new_v4();

    let create_response = {
        let display_name = "sysmon-generator";
        let artifact = get_sysmon_generator()?;
        let metadata = PluginMetadata {
            tenant_id: tenant_id.clone(),
            display_name: display_name.to_owned(),
            plugin_type: PluginType::Generator,
            event_source_id: Some(event_source_id.clone()),
        };

        client
            .create_plugin(
                metadata,
                futures::stream::once(async move { artifact.clone() }),
            )
            .timeout(std::time::Duration::from_secs(5))
            .await??
    };

    let plugin_id = create_response.plugin_id;

    let request = DeployPluginRequest { plugin_id };

    let _response = client
        .deploy_plugin(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    Ok(())
}

fn assert_contains(input: &str, expected_substr: &str) {
    assert!(
        input.contains(expected_substr),
        "Expected input '{input}' to contain '{expected_substr}'"
    )
}

#[test_log::test(tokio::test)]
/// So we *expect* this call to fail since it's an arbitrary PluginID that
/// hasn't been created yet
async fn test_deploy_plugin_but_plugin_id_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PluginRegistryServiceClient::from_env().await?;

    let randomly_selected_plugin_id = uuid::Uuid::new_v4();

    let request = DeployPluginRequest {
        plugin_id: randomly_selected_plugin_id,
    };

    let response = client
        .deploy_plugin(request)
        .timeout(std::time::Duration::from_secs(5))
        .await?;

    match response {
        Err(PluginRegistryServiceClientError::ErrorStatus(s)) => {
            // TODO: We should consider a dedicated "PluginIDDoesntExist" exception
            assert_contains(s.message(), "Failed to operate on postgres");
        }
        _ => panic!("Expected an error"),
    };
    Ok(())
}
