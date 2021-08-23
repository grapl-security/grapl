#![cfg(feature = "integration")]
// ^ Marks the entire file, including helpers, as "only compile for integration tests"

use model_plugin_deployer::client::ModelPluginDeployerRpcClient;
use model_plugin_deployer::client::DeployModelRequest;

#[tokio::test]
async fn test_deploy_model() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ModelPluginDeployerRpcClient::from_env().await?;
    let request = tonic::Request::new(DeployModelRequest {
        schema_type: 1,
        schema: b"Hello".to_vec(),
    });
    let _response = client.deploy_model(request);
    Ok(())
}


#[tokio::test]
async fn test_unsupported_schema_type() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ModelPluginDeployerRpcClient::from_env().await?;
    let request = tonic::Request::new(DeployModelRequest {
        schema_type: 0,
        schema: b"Hello".to_vec(),
    });
    let response = client.deploy_model(request).await?;
    println!("RESPONSE = {:?}", response);
    Ok(())
}