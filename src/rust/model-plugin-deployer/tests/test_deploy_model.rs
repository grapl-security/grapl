#![cfg(feature = "integration")]

use model_plugin_deployer::client::{
    DeployModelRequest,
    RpcClient,
    SchemaType,
};
use tonic::Code;

/// For now, this is just a smoke test. This test can and should evolve as
/// the service matures.
#[tokio::test]
async fn test_deploy_model() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RpcClient::from_env().await?;
    let request = tonic::Request::new(DeployModelRequest {
        schema_type: SchemaType::Graphql.into(),
        schema: b"Hello".to_vec(),
    });
    client.deploy_model(request).await?;
    Ok(())
}

#[tokio::test]
async fn test_unsupported_schema_type() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RpcClient::from_env().await?;
    let request = tonic::Request::new(DeployModelRequest {
        schema_type: SchemaType::Unspecified.into(),
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
