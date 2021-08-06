use tonic::{transport::Server, Request, Response, Status};

use crate::{{cookiecutter.snake_project_name}}::{{cookiecutter.service_name}}Request;
pub use crate::{{cookiecutter.snake_project_name}}::{{cookiecutter.service_name}}Response;
pub use crate::{{cookiecutter.snake_project_name}}::{{cookiecutter.snake_project_name}}_rpc_server::{{cookiecutter.service_name}}Rpc;
pub use crate::{{cookiecutter.snake_project_name}}::{{cookiecutter.snake_project_name}}_rpc_server::{{cookiecutter.service_name}}RpcServer;

#[derive(Default)]
pub struct {{cookiecutter.service_name}} {
    // Right now this struct just exists so we can attach behaviors to it.
    // If you need state later, you can add it.
}

#[tonic::async_trait]
impl {{cookiecutter.service_name}}Rpc for {{cookiecutter.service_name}} {

    #[tracing::instrument(
        source_addr = request.remote_addr(),
        client_id = request.get_ref().request_meta.client_id,
        skip(self, request),
    )]
    async fn handle_request(
        &self,
        _request: Request<{{cookiecutter.service_name}}Request>,
    ) -> Result<Response<{{cookiecutter.service_name}}Response>, Status> {
        let start = quanta::Instant::now();

        let reply = {{cookiecutter.service_name}}Response {

        };

        let delta = quanta::Instant::now().duration_since(start);
        metrics::histogram!("request_ns", delta);

        Ok(Response::new(reply))
    }
}

pub async fn exec_service()  -> Result<(), Box<dyn std::error::Error>> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
    .set_serving::<{{cookiecutter.service_name}}RpcServer<{{cookiecutter.service_name}}>>()
    .await;

    let addr = "[::1]:50051".parse().unwrap();
    let {{cookiecutter.snake_project_name}}_instance = {{cookiecutter.service_name}}::default();

    tracing::info!(
        message="HealthServer + {{cookiecutter.service_name}} listening",
        addr=?addr,
    );

    metrics::register_counter!("request_count", "count of requests made to endpoint");
    metrics::register_histogram!("request_ns", "nanoseconds for request execution");


    Server::builder()
        .add_service(health_service)
        .add_service({{cookiecutter.service_name}}RpcServer::new({{cookiecutter.snake_project_name}}_instance))
        .serve(addr)
        .await?;

    Ok(())
}
