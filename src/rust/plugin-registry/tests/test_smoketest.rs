#![cfg(feature = "integration")]

use plugin_registry::client::PluginRegistryServiceClient;
use rust_proto::plugin_registry::GetGeneratorsForEventSourceRequest;

/// For now, this is just a smoke test. This test can and should evolve as
/// the service matures.
#[tokio::test]
async fn test_smoke_test_create_client() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PluginRegistryServiceClient::from_env().await?;
    let request = GetGeneratorsForEventSourceRequest {
        event_source_id: uuid::Uuid::new_v4(),
    };
    client.get_generators_for_event_source(request).await?;
    Ok(())
}
