#![cfg(feature = "integration")]

use grapl_utils::future_ext::GraplFutureExt;
use plugin_registry::client::PluginRegistryServiceClient;
use rust_proto::plugin_registry::DeployPluginRequest;

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
