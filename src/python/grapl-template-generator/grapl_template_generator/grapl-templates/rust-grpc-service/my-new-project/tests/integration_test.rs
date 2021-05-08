mod common;

use common::ServiceContext;
use test_context::futures;

use my_new_project::client::MyNewProjectRpcClient;
use my_new_project::client::MyNewProjectRequest;
use my_new_project::client::Channel;
use my_new_project::client::Timeout;

use std::time::Duration;

#[test_context::test_context(ServiceContext)]
#[tokio::test]
async fn smoketest(_ctx: &mut ServiceContext) -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://[::1]:50051").connect().await?;

    let timeout_channel = Timeout::new(channel, Duration::from_millis(1000));

    let mut client = MyNewProjectRpcClient::new(timeout_channel);

    let request = tonic::Request::new(MyNewProjectRequest {});

    let _response = client.handle_request(request).await?;
    panic!("This test could use some work!");
}
