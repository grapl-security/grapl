#![cfg(feature = "integration")]

use grapl_utils::future_ext::GraplFutureExt;
use plugin_registry::client::{
    PluginRegistryServiceClient,
    PluginRegistryServiceClientError,
};
use rust_proto_new::graplinc::grapl::api::plugin_registry::v1beta1::{
    CreatePluginRequest,
    DeployPluginRequest,
    PluginType,
};

#[test_log::test(tokio::test)]
async fn test_deploy_plugin() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PluginRegistryServiceClient::from_env().await?;

    let tenant_id = uuid::Uuid::new_v4();

    let create_response = {
        let display_name = uuid::Uuid::new_v4().to_string();
        let request = CreatePluginRequest {
            plugin_artifact: b"dummy vec for now".to_vec(),
            tenant_id: tenant_id.clone(),
            display_name: display_name.clone(),
            plugin_type: PluginType::Generator,
        };

        client
            .create_plugin(request)
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
/// So we *expect* this call to fail since it's an arbitrary PluginID that
/// hasn't been created yet
async fn test_deploy_plugin_but_random_plugin_id() -> Result<(), Box<dyn std::error::Error>> {
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
            assert!(s.message().contains("Failed to operate on postgres"));
        }
        _ => panic!("Expected an error"),
    };
    Ok(())
}
