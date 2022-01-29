#![cfg(feature = "integration")]

use plugin_registry::nomad::{
    client::{
        NomadClient,
    },
};

#[test_log::test(tokio::test)]
async fn test_nomad_client_create_namespace() -> Result<(), Box<dyn std::error::Error>> {
    let client = NomadClient::from_env();
    client
        .create_namespace("test-nomad-client-create-namespace")
        .await?;
    Ok(())
}