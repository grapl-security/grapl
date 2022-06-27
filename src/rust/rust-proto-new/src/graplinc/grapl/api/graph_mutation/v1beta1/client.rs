use crate::{
    graplinc::grapl::api::graph_mutation::v1beta1::messages::{
        CreateEdgeRequest,
        CreateEdgeResponse,
        CreateNodeRequest,
        CreateNodeResponse,
        SetNodePropertyRequest,
        SetNodePropertyResponse,
    },
    protobufs::graplinc::grapl::api::graph_mutation::v1beta1::{
        graph_mutation_service_client::GraphMutationServiceClient as GraphMutationServiceClientProto,
        CreateEdgeRequest as CreateEdgeRequestProto,
        CreateNodeRequest as CreateNodeRequestProto,
        SetNodePropertyRequest as SetNodePropertyRequestProto,
    },
    protocol::status::Status,
    SerDeError,
};

#[derive(thiserror::Error, Debug)]
pub enum GraphMutationClientError {
    #[error("Failed to deserialize response {0}")]
    SerDeError(#[from] SerDeError),
    #[error("Status {0}")]
    Status(#[from] Status),
    #[error("ConnectError")]
    ConnectError(tonic::transport::Error),
}

#[derive(Clone)]
pub struct GraphMutationClient {
    inner: GraphMutationServiceClientProto<tonic::transport::Channel>,
}

impl GraphMutationClient {
    pub async fn connect<T>(endpoint: T) -> Result<Self, GraphMutationClientError>
    where
        T: TryInto<tonic::transport::Endpoint>,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        Ok(GraphMutationClient {
            inner: GraphMutationServiceClientProto::connect(endpoint)
                .await
                .map_err(GraphMutationClientError::ConnectError)?,
        })
    }

    pub async fn create_node(
        &mut self,
        request: CreateNodeRequest,
    ) -> Result<CreateNodeResponse, GraphMutationClientError> {
        let raw_request: CreateNodeRequestProto = request.into();
        let raw_response = self
            .inner
            .create_node(raw_request)
            .await
            .map_err(Status::from)?;
        let response = raw_response.into_inner().try_into()?;
        Ok(response)
    }

    pub async fn set_node_property(
        &mut self,
        request: SetNodePropertyRequest,
    ) -> Result<SetNodePropertyResponse, GraphMutationClientError> {
        let raw_request: SetNodePropertyRequestProto = request.into();
        let raw_response = self
            .inner
            .set_node_property(raw_request)
            .await
            .map_err(Status::from)?;
        let response = raw_response.into_inner().try_into()?;
        Ok(response)
    }

    pub async fn create_edge(
        &mut self,
        request: CreateEdgeRequest,
    ) -> Result<CreateEdgeResponse, GraphMutationClientError> {
        let raw_request: CreateEdgeRequestProto = request.into();
        let raw_response = self
            .inner
            .create_edge(raw_request)
            .await
            .map_err(Status::from)?;
        let response = raw_response.into_inner().try_into()?;
        Ok(response)
    }
}
