#![cfg(feature = "integration")]

use grapl_utils::future_ext::GraplFutureExt;
use plugin_registry::client::PluginRegistryServiceClient;
use rust_proto::plugin_registry::DeployPluginRequest;

#[test_log::test(tokio::test)]
async fn test_deploy_plugin() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PluginRegistryServiceClient::from_env().await?;

    let plugin_id = uuid::Uuid::new_v4();

    let request = DeployPluginRequest { plugin_id };

    let _response = client
        .deploy_plugin(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;
    Ok(())
}
