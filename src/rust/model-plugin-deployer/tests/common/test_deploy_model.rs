#![cfg(feature = "integration")]
// ^ Marks the entire file, including helpers, as "only compile for integration tests"

use model_plugin_deployer::client::ModelPluginDeployerRpcClient;
use crate::client::DeployModelRequest;

#[tokio::test]
async fn test_deploy_model() {
    let client = ModelPluginDeployerRpcClient::new();
    let request = tonic::Request::new(DeployModelRequest {
    });
    let _response = client.deploy_model(request);
}