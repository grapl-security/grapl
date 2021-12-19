#![cfg(feature = "integration")]

use grapl_utils::future_ext::GraplFutureExt;
use plugin_registry::client::PluginRegistryServiceClient;
use rust_proto::plugin_registry::{
    CreatePluginRequest,
    GetPluginRequest,
    GetPluginResponse,
    PluginType,
};

/// For now, this is just a smoke test. This test can and should evolve as
/// the service matures.
#[test_log::test(tokio::test)]
async fn test_smoke_test_create_client() -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!(
        env=?std::env::args(),
    );
    let mut client = PluginRegistryServiceClient::from_env().await?;

    let tenant_id = uuid::Uuid::new_v4();

    let display_name = uuid::Uuid::new_v4().to_string();
    let request = CreatePluginRequest {
        plugin_artifact: b"dummy vec for now".to_vec(),
        tenant_id: tenant_id.clone(),
        display_name: display_name.clone(),
        plugin_type: PluginType::Generator,
    };

    let response = client
        .create_plugin(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;
    let plugin_id = response.plugin_id;

    let get_response: GetPluginResponse = client
        .get_plugin(GetPluginRequest {
            plugin_id,
            tenant_id,
        })
        .timeout(std::time::Duration::from_secs(5))
        .await??;
    assert_eq!(get_response.plugin.plugin_id, plugin_id);
    assert_eq!(get_response.plugin.plugin_type, PluginType::Generator);
    assert_eq!(get_response.plugin.display_name, display_name);
    Ok(())
}
