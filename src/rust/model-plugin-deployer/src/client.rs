// Re-export for public consumption
pub use tonic::transport::Channel;
pub use tower::timeout::Timeout;

pub use crate::model_plugin_deployer::{
    model_plugin_deployer_rpc_client::ModelPluginDeployerRpcClient,
    DeployModelRequest,
};

/*
If you want to provide a higher-level client abstraction - like
a ModelPluginDeployerClient that hides the grpc implementation details -
this would be the place to add that.
*/
