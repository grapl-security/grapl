use std::time::Duration;

pub use tonic::transport::Channel;
use tonic::transport::Endpoint;
pub use tower::timeout::Timeout;

// Re-export for public consumption
pub use crate::model_plugin_deployer::{
    model_plugin_deployer_rpc_client::ModelPluginDeployerRpcClient,
    DeployModelRequest,
    DeployModelResponse,
};

/*
USe this file to add higher-level client abstractions to ModelPluginDeployerRpcClient.
*/

impl ModelPluginDeployerRpcClient<Channel> {
    #![allow(dead_code)]
    pub async fn from_env() -> Result<ModelPluginDeployerRpcClient<Channel>, Box<dyn std::error::Error>> {
        let host = std::env::var("GRAPL_MODEL_PLUGIN_DEPLOYER_V2_HOST").expect("GRAPL_MODEL_PLUGIN_DEPLOYER_V2_HOST");
        let port = std::env::var("GRAPL_MODEL_PLUGIN_DEPLOYER_V2_PORT").expect("GRAPL_MODEL_PLUGIN_DEPLOYER_V2_PORT");
        let endpoint_str = format!("http://{}:{}", host, port);
        let endpoint = Endpoint::from_shared(endpoint_str)?
            .timeout(Duration::from_secs(5))
            .concurrency_limit(30);
        let channel = endpoint.connect().await?;
        Ok(ModelPluginDeployerRpcClient::new(channel.clone()))
    }
}