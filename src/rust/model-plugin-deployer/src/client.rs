use std::time::Duration;

pub use tonic::transport::Channel;
use tonic::transport::Endpoint;
pub use tower::timeout::Timeout;

// Re-export for public consumption
pub use crate::model_plugin_deployer::{
    model_plugin_deployer_rpc_service_client::ModelPluginDeployerRpcServiceClient as RpcClient,
    DeployModelRequest,
    DeployModelResponse,
    SchemaType,
};

/*
USe this file to add higher-level client abstractions to RpcClient.
*/

const HOST_ENV_VAR: &'static str = "GRAPL_MODEL_PLUGIN_DEPLOYER_HOST";
const PORT_ENV_VAR: &'static str = "GRAPL_MODEL_PLUGIN_DEPLOYER_PORT";

impl RpcClient<Channel> {
    #![allow(dead_code)]
    pub async fn from_env() -> Result<RpcClient<Channel>, Box<dyn std::error::Error>> {
        let host = std::env::var(HOST_ENV_VAR).expect(HOST_ENV_VAR);
        let port = std::env::var(PORT_ENV_VAR).expect(PORT_ENV_VAR);
        RpcClient::from_endpoint(host, port).await
    }

    pub async fn from_endpoint(
        host: String,
        port: String,
    ) -> Result<RpcClient<Channel>, Box<dyn std::error::Error>> {
        let endpoint_str = format!("http://{}:{}", host, port);

        // TODO: It might make sense to make these values configurable.
        let endpoint = Endpoint::from_shared(endpoint_str)?
            .timeout(Duration::from_secs(5))
            .concurrency_limit(30);
        let channel = endpoint.connect().await?;
        Ok(RpcClient::new(channel))
    }
}
