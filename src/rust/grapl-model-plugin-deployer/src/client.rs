use std::time::Duration;

pub use tonic::transport::Channel;
use tonic::transport::Endpoint;
pub use tower::timeout::Timeout;

use crate::grapl_model_plugin_deployer_proto::{
    grapl_model_plugin_deployer_rpc_client::GraplModelPluginDeployerRpcClient,
    grapl_model_plugin_deployer_rpc_server::GraplModelPluginDeployerRpcServer,
    GraplModelPluginDeployerRequest,
    GraplModelPluginDeployerResponse,
    GraplRequestMeta,
};

#[derive(thiserror::Error, Debug)]
pub enum ModelPluginDeployerClientError {
    #[error("SchemaVersionConflict")]
    SchemaVersionConflict { error_message: String },
    #[error("TransportError")]
    TransportError(#[from] tonic::transport::Error),
    #[error("GrpcErrorStatus")]
    GrpcErrorStatus(#[from] tonic::Status),
}

#[derive(Clone)]
pub struct ModelPluginDeployerClient {
    client_name: String,
    client: GraplModelPluginDeployerRpcClient<tower::timeout::Timeout<tonic::transport::Channel>>,
}

impl ModelPluginDeployerClient {
    pub async fn new(client_name: String, client_list: impl Iterator<Item = Endpoint>) -> Self {
        let channel = Channel::balance_list(client_list);

        let timeout_channel = Timeout::new(channel, Duration::from_millis(3000));

        ModelPluginDeployerClient {
            client_name,
            client: GraplModelPluginDeployerRpcClient::new(timeout_channel),
        }
    }

    pub async fn deploy_plugin(
        &mut self,
        model_plugin_schema: String,
        schema_version: u32,
    ) -> Result<(), ModelPluginDeployerClientError> {
        let response = self
            .client
            .deploy_plugin(GraplModelPluginDeployerRequest {
                request_meta: Some(self.request_meta()),
                model_plugin_schema,
                schema_version,
            })
            .await?;
        Ok(())
    }

    fn request_meta(&self) -> GraplRequestMeta {
        GraplRequestMeta {
            request_id: uuid::Uuid::new_v4().to_string(),
            client_name: self.client_name.to_string(),
        }
    }
}
