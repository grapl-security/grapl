#![cfg(feature = "integration")]

use plugin_registry::client::PluginRegistryServiceClient;
use tonic::Code;

/// For now, this is just a smoke test. This test can and should evolve as
/// the service matures.
#[tokio::test]
async fn test_smoke_test_create_client() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PluginRegistryServiceClient::from_env().await?;
    Ok(())
}
