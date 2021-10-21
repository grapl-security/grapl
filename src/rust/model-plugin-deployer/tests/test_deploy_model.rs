#![cfg(feature = "integration")]
// ^ Marks the entire file, including helpers, as "only compile for integration tests"

use model_plugin_deployer::client::{
    DeployModelRequest,
    RpcClient,
};
use tonic::Code;

#[tokio::test]
async fn test_deploy_model() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RpcClient::from_env().await?;
    let request = tonic::Request::new(DeployModelRequest {
        schema_type: 1,
        schema: b"Hello".to_vec(),
    });
    client.deploy_model(request).await?;
    Ok(())
}

#[tokio::test]
async fn test_unsupported_schema_type() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RpcClient::from_env().await?;
    let request = tonic::Request::new(DeployModelRequest {
        schema_type: 0,
        schema: b"Hello".to_vec(),
    });
    let result = client.deploy_model(request).await;
    match result {
        Ok(_) => Err("Unexpected OK".into()),
        Err(status) => {
            assert_eq!(status.code(), Code::InvalidArgument);
            assert_eq!(status.message(), "Unhandled schema type");
            Ok(())
        }
    }
}
