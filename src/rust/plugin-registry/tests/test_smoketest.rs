#![cfg(feature = "integration")]

use grapl_utils::future_ext::GraplFutureExt;
use plugin_registry::client::PluginRegistryServiceClient;
use rust_proto::plugin_registry::{CreatePluginRequest, PluginType};

/// For now, this is just a smoke test. This test can and should evolve as
/// the service matures.
#[test_log::test(tokio::test)]
async fn test_smoke_test_create_client() -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!(
        message="test_smoke_test_create_client",
        env=?std::env::args(),
    );
    let mut client = PluginRegistryServiceClient::from_env().await?;

    let request = CreatePluginRequest {
        plugin_artifact: b"dummy vec for now".to_vec(),
        tenant_id: uuid::Uuid::new_v4(),
        display_name: "Super Cool Plugin haha!!!".to_string(),
        plugin_type: PluginType::Generator,
    };

    client
        .create_plugin(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;
    Ok(())
}
