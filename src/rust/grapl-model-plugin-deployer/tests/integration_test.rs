mod common;

use common::ServiceContext;
use test_context::futures;

use grapl_model_plugin_deployer::client::GraplModelPluginDeployerRpcClient;
use grapl_model_plugin_deployer::client::GraplModelPluginDeployerRequest;
use grapl_model_plugin_deployer::client::Channel;
use grapl_model_plugin_deployer::client::Timeout;

use std::time::Duration;

#[test_context::test_context(ServiceContext)]
#[tokio::test]
async fn smoketest(_ctx: &mut ServiceContext) -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://[::1]:50051").connect().await?;

    let timeout_channel = Timeout::new(channel, Duration::from_millis(1000));

    let mut client = GraplModelPluginDeployerRpcClient::new(timeout_channel);

    let request = tonic::Request::new(GraplModelPluginDeployerRequest {});

    let _response = client.handle_request(request).await?;
    panic!("This test could use some work!");
}
