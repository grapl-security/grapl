use crate::{
    graplinc::grapl::api::scylla_provisioner::v1beta1::messages::{
        DeployGraphSchemasRequest,
        DeployGraphSchemasResponse,
    },
    protobufs::graplinc::grapl::api::scylla_provisioner::v1beta1::{
        scylla_provisioner_service_client::ScyllaProvisionerServiceClient,
        DeployGraphSchemasRequest as DeployGraphSchemasRequestProto,
    },
    protocol::status::Status,
    SerDeError,
};

#[derive(thiserror::Error, Debug)]
pub enum ScyllaProvisionerClientError {
    #[error("Failed to deserialize response {0}")]
    SerDeError(#[from] SerDeError),
    #[error("Status {0}")]
    Status(#[from] Status),
    #[error("ConnectError")]
    ConnectError(tonic::transport::Error),
}

#[derive(Clone)]
pub struct ScyllaProvisionerClient {
    inner: ScyllaProvisionerServiceClient<tonic::transport::Channel>,
}

impl ScyllaProvisionerClient {
    pub async fn connect<T>(endpoint: T) -> Result<Self, ScyllaProvisionerClientError>
    where
        T: TryInto<tonic::transport::Endpoint>,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        Ok(ScyllaProvisionerClient {
            inner: ScyllaProvisionerServiceClient::connect(endpoint)
                .await
                .map_err(ScyllaProvisionerClientError::ConnectError)?,
        })
    }

    pub async fn query_graph_with_uid(
        &mut self,
        request: DeployGraphSchemasRequest,
    ) -> Result<DeployGraphSchemasResponse, ScyllaProvisionerClientError> {
        let request: DeployGraphSchemasRequestProto = request.into();
        Ok(self
            .inner
            .deploy_graph_schemas(request)
            .await
            .map_err(Status::from)?
            .into_inner()
            .try_into()?)
    }
}
