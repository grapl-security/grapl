#![cfg(feature = "new_integration_tests")]

use grapl_utils::future_ext::GraplFutureExt;
use plugin_registry::client::FromEnv;
use rust_proto_new::graplinc::grapl::api::plugin_registry::v1beta1::{
    CreatePluginRequestMetadata,
    GetPluginRequest,
    GetPluginResponse,
    PluginRegistryServiceClient,
    PluginType,
};

/// For now, this is just a smoke test. This test can and should evolve as
/// the service matures.
#[test_log::test(tokio::test)]
async fn test_create_plugin() -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!(
        env=?std::env::args(),
    );
    let mut client = PluginRegistryServiceClient::from_env().await?;

    let tenant_id = uuid::Uuid::new_v4();

    let display_name = uuid::Uuid::new_v4().to_string();

    let meta = CreatePluginRequestMetadata {
        tenant_id: tenant_id.clone(),
        display_name: display_name.clone(),
        plugin_type: PluginType::Generator,
    };

    let single_chunk = b"dummy vec for now".to_vec();

    let response = client
        .create_plugin(meta, single_chunk.into_iter())
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
