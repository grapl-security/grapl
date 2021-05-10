use tonic::{transport::Server, Request, Response, Status};

use crate::grapl_model_plugin_deployer::GraplModelPluginDeployerRequest;
pub use crate::grapl_model_plugin_deployer::GraplModelPluginDeployerResponse;
pub use crate::grapl_model_plugin_deployer::grapl_model_plugin_deployer_rpc_server::GraplModelPluginDeployerRpc;
pub use crate::grapl_model_plugin_deployer::grapl_model_plugin_deployer_rpc_server::GraplModelPluginDeployerRpcServer;

#[derive(Default)]
pub struct GraplModelPluginDeployer {}

#[tonic::async_trait]
impl GraplModelPluginDeployerRpc for GraplModelPluginDeployer {
    async fn handle_request(
        &self,
        request: Request<GraplModelPluginDeployerRequest>,
    ) -> Result<Response<GraplModelPluginDeployerResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = GraplModelPluginDeployerResponse {

        };
        Ok(Response::new(reply))
    }
}

pub async fn exec_service()  -> Result<(), Box<dyn std::error::Error>> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
    .set_serving::<GraplModelPluginDeployerRpcServer<GraplModelPluginDeployer>>()
    .await;

    let addr = "[::1]:50051".parse().unwrap();
    let my_new_project_instance = GraplModelPluginDeployer::default();

    tracing::info!(
    message="HealthServer + GraplModelPluginDeployer listening",
    addr=?addr,
    );

    Server::builder()
    .add_service(health_service)
    .add_service(GraplModelPluginDeployerRpcServer::new(my_new_project_instance))
    .serve(addr)
    .await?;

    Ok(())
}
