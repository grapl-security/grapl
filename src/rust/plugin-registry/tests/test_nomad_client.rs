#![cfg(feature = "integration")]

use plugin_registry::nomad_client;

/// For now, this is just a smoke test. This test can and should evolve as
/// the service matures.
#[test_log::test(tokio::test)]
async fn test_nomad_client() -> Result<(), Box<dyn std::error::Error>> {
    let client = nomad_client::NomadClient::from_env();
    client.create_namespace().await?;
    Ok(())
}
