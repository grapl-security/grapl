#![cfg(feature = "integration")]

use plugin_registry::nomad_client;

#[test_log::test(tokio::test)]
async fn test_nomad_client() -> Result<(), Box<dyn std::error::Error>> {
    let client = nomad_client::NomadClient::from_env();
    client.create_namespace("hello").await?;
    Ok(())
}
