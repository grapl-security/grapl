mod common;

use common::ServiceContext;
use test_context::futures;

use {{cookiecutter.snake_project_name}}::client::{{cookiecutter.service_name}}RpcClient;
use {{cookiecutter.snake_project_name}}::client::{{cookiecutter.service_name}}Request;
use {{cookiecutter.snake_project_name}}::client::Channel;
use {{cookiecutter.snake_project_name}}::client::Timeout;
use {{cookiecutter.snake_project_name}}::{{cookiecutter.snake_project_name}}::get_socket_addr;

use std::time::Duration;

#[test_context::test_context(ServiceContext)]
#[tokio::test]
async fn smoketest(_ctx: &mut ServiceContext) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = format!("http://{}", get_socket_addr());
    let channel = Channel::from_shared(endpoint)?.connect().await?;

    let timeout_channel = Timeout::new(channel, Duration::from_millis(1000));

    let mut client = {{cookiecutter.service_name}}RpcClient::new(timeout_channel);

    let request = tonic::Request::new({{cookiecutter.service_name}}Request {
    });

    let _response = client.handle_request(request).await?;
    todo!("This test could use some work!");
}
