use tonic::{transport::Server, Request, Response, Status};

use crate::my_new_project::MyNewProjectRequest;
pub use crate::my_new_project::MyNewProjectResponse;
pub use crate::my_new_project::my_new_project_rpc_server::MyNewProjectRpc;
pub use crate::my_new_project::my_new_project_rpc_server::MyNewProjectRpcServer;

#[derive(Default)]
pub struct MyNewProject {}

#[tonic::async_trait]
impl MyNewProjectRpc for MyNewProject {
    async fn handle_request(
        &self,
        request: Request<MyNewProjectRequest>,
    ) -> Result<Response<MyNewProjectResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = MyNewProjectResponse {

        };
        Ok(Response::new(reply))
    }
}

pub async fn exec_service()  -> Result<(), Box<dyn std::error::Error>> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
    .set_serving::<MyNewProjectRpcServer<MyNewProject>>()
    .await;

    let addr = "[::1]:50051".parse().unwrap();
    let my_new_project_instance = MyNewProject::default();

    tracing::info!(
    message="HealthServer + MyNewProject listening",
    addr=?addr,
    );

    Server::builder()
    .add_service(health_service)
    .add_service(MyNewProjectRpcServer::new(my_new_project_instance))
    .serve(addr)
    .await?;

    Ok(())
}
