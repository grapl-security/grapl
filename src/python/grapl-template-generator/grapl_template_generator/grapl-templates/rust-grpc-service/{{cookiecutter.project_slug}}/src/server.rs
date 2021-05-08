use tonic::{transport::Server, Request, Response, Status};

use crate::{{cookiecutter.snake_project_name}}::{{cookiecutter.service_name}}Request;
pub use crate::{{cookiecutter.snake_project_name}}::{{cookiecutter.service_name}}Response;
pub use crate::{{cookiecutter.snake_project_name}}::{{cookiecutter.snake_project_name}}_rpc_server::{{cookiecutter.service_name}}Rpc;
pub use crate::{{cookiecutter.snake_project_name}}::{{cookiecutter.snake_project_name}}_rpc_server::{{cookiecutter.service_name}}RpcServer;

#[derive(Default)]
pub struct {{cookiecutter.service_name}} {}

#[tonic::async_trait]
impl {{cookiecutter.service_name}}Rpc for {{cookiecutter.service_name}} {
    async fn handle_request(
        &self,
        request: Request<{{cookiecutter.service_name}}Request>,
    ) -> Result<Response<{{cookiecutter.service_name}}Response>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = {{cookiecutter.service_name}}Response {

        };
        Ok(Response::new(reply))
    }
}

pub async fn exec_service()  -> Result<(), Box<dyn std::error::Error>> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
    .set_serving::<{{cookiecutter.service_name}}RpcServer<{{cookiecutter.service_name}}>>()
    .await;

    let addr = "[::1]:50051".parse().unwrap();
    let my_new_project_instance = {{cookiecutter.service_name}}::default();

    tracing::info!(
    message="HealthServer + {{cookiecutter.service_name}} listening",
    addr=?addr,
    );

    Server::builder()
    .add_service(health_service)
    .add_service({{cookiecutter.service_name}}RpcServer::new(my_new_project_instance))
    .serve(addr)
    .await?;

    Ok(())
}
