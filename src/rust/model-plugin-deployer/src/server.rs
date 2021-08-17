use std::net::SocketAddr;

use tonic::{
    transport::Server,
    Code,
    Request,
    Response,
    Status,
};

pub use crate::model_plugin_deployer::{
    model_plugin_deployer_rpc_server::{
        ModelPluginDeployerRpc,
        ModelPluginDeployerRpcServer,
    },
    DeployModelResponse,
};
use crate::model_plugin_deployer::{
    DeployModelRequest,
    SchemaType,
};

#[derive(Default)]
pub struct ModelPluginDeployer {
    // Right now this struct just exists so we can attach behaviors to it.
// If you need state later, you can add it.
}

#[tonic::async_trait]
impl ModelPluginDeployerRpc for ModelPluginDeployer {
    #[tracing::instrument(
        source_addr = request.remote_addr(),
        client_id = request.get_ref().request_meta.client_id,
        skip(self, request),
    )]
    async fn deploy_model(
        &self,
        request: Request<DeployModelRequest>,
    ) -> Result<Response<DeployModelResponse>, Status> {
        let start = quanta::Instant::now();

        let message = request.into_inner();
        match SchemaType::from_i32(message.schema_type) {
            Some(SchemaType::Graphql) => {
                // Read the schema as graphql
            }
            _ => return Err(Status::new(Code::InvalidArgument, "Unhandled schema type")),
        }

        let reply = DeployModelResponse {};

        let delta = quanta::Instant::now().duration_since(start);
        metrics::histogram!("request_ns", delta);

        Ok(Response::new(reply))
    }
}

pub async fn exec_service(socket_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<ModelPluginDeployerRpcServer<ModelPluginDeployer>>()
        .await;

    let model_plugin_deployer_instance = ModelPluginDeployer::default();

    metrics::register_counter!("request_count", "count of requests made to endpoint");
    metrics::register_histogram!("request_ns", "nanoseconds for request execution");

    tracing::info!(
        message="About to start ModelPluginDeployer + HealthServer",
        addr=?socket_addr,
    );

    Server::builder()
        .add_service(health_service)
        .add_service(ModelPluginDeployerRpcServer::new(
            model_plugin_deployer_instance,
        ))
        .serve(socket_addr)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deploy_model_validation() -> Result<(), String> {
        let service_instance = ModelPluginDeployer::default();
        let inner_request = DeployModelRequest::default();
        let outer_request = Request::new(inner_request);
        let response = service_instance.deploy_model(outer_request).await;
        match response {
            Ok(_) => Err("Unexpected OK".into()),
            Err(status) => {
                assert_eq!(status.code(), Code::InvalidArgument);
                assert_eq!(status.message(), "Unhandled schema type");
                Ok(())
            }
        }
    }
}
