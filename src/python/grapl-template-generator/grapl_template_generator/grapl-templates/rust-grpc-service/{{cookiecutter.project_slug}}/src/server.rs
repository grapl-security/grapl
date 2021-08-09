use tonic::{transport::Server, Request, Response, Status};
use std::net::SocketAddr;

use crate::{{cookiecutter.snake_project_name}}::{{cookiecutter.crate_name}}Request;
pub use crate::{{cookiecutter.snake_project_name}}::{{cookiecutter.crate_name}}Response;
pub use crate::{{cookiecutter.snake_project_name}}::{{cookiecutter.snake_project_name}}_rpc_server::{{cookiecutter.crate_name}}Rpc;
pub use crate::{{cookiecutter.snake_project_name}}::{{cookiecutter.snake_project_name}}_rpc_server::{{cookiecutter.crate_name}}RpcServer;

#[derive(Default)]
pub struct {{cookiecutter.crate_name}} {
    // Right now this struct just exists so we can attach behaviors to it.
    // If you need state later, you can add it.
}

#[tonic::async_trait]
impl {{cookiecutter.crate_name}}Rpc for {{cookiecutter.crate_name}} {

    #[tracing::instrument(
        source_addr = request.remote_addr(),
        client_id = request.get_ref().request_meta.client_id,
        skip(self, request),
    )]
    async fn handle_request(
        &self,
        request: Request<{{cookiecutter.crate_name}}Request>,
    ) -> Result<Response<{{cookiecutter.crate_name}}Response>, Status> {
        // Prevents a dead-code error.
        // Remove this when you actually do something with this code. 
        let _dummy = request; // or whatever field a request might have

        let start = quanta::Instant::now();

        let reply = {{cookiecutter.crate_name}}Response {

        };

        let delta = quanta::Instant::now().duration_since(start);
        metrics::histogram!("request_ns", delta);

        Ok(Response::new(reply))
    }
}

pub async fn exec_service(socket_addr: SocketAddr)  -> Result<(), Box<dyn std::error::Error>> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
    .set_serving::<{{cookiecutter.crate_name}}RpcServer<{{cookiecutter.crate_name}}>>()
    .await;

    let {{cookiecutter.snake_project_name}}_instance = {{cookiecutter.crate_name}}::default();

    metrics::register_counter!("request_count", "count of requests made to endpoint");
    metrics::register_histogram!("request_ns", "nanoseconds for request execution");

    tracing::info!(
        message="About to start {{cookiecutter.crate_name}} + HealthServer",
        addr=?socket_addr,
    );

    Server::builder()
        .add_service(health_service)
        .add_service({{cookiecutter.crate_name}}RpcServer::new({{cookiecutter.snake_project_name}}_instance))
        .serve(socket_addr)
        .await?;

    Ok(())
}
