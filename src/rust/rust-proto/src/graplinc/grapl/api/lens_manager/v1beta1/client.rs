use crate::{
    graplinc::grapl::api::lens_manager::v1beta1::messages::{
        AddNodeToScopeRequest,
        AddNodeToScopeResponse,
        CloseLensRequest,
        CloseLensResponse,
        CreateLensRequest,
        CreateLensResponse,
        MergeLensRequest,
        MergeLensResponse,
        RemoveNodeFromAllScopesRequest,
        RemoveNodeFromAllScopesResponse,
        RemoveNodeFromScopeRequest,
        RemoveNodeFromScopeResponse,
    },
    protobufs::graplinc::grapl::api::lens_manager::v1beta1::{
        lens_manager_service_client::LensManagerServiceClient as LensManagerServiceClientProto,
        AddNodeToScopeRequest as AddNodeToScopeRequestProto,
        CloseLensRequest as CloseLensRequestProto,
        CreateLensRequest as CreateLensRequestProto,
        MergeLensRequest as MergeLensRequestProto,
        RemoveNodeFromAllScopesRequest as RemoveNodeFromAllScopesRequestProto,
        RemoveNodeFromScopeRequest as RemoveNodeFromScopeRequestProto,
    },
    protocol::status::Status,
    SerDeError,
};

#[derive(thiserror::Error, Debug)]
pub enum LensManagerServiceClientError {
    #[error("Failed to deserialize response {0}")]
    SerDeError(#[from] SerDeError),
    #[error("Status {0}")]
    Status(Status),
    #[error("ConnectError")]
    ConnectError(tonic::transport::Error),
}

#[derive(Clone)]
pub struct LensManagerServiceClient {
    inner: LensManagerServiceClientProto<tonic::transport::Channel>,
}

impl LensManagerServiceClient {
    pub async fn connect<T>(endpoint: T) -> Result<Self, LensManagerServiceClientError>
    where
        T: TryInto<tonic::transport::Endpoint>,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        Ok(LensManagerServiceClient {
            inner: LensManagerServiceClientProto::connect(endpoint)
                .await
                .map_err(LensManagerServiceClientError::ConnectError)?,
        })
    }

    /// Creates a new lens with an empty scope
    pub async fn create_lens(
        &mut self,
        request: CreateLensRequest,
    ) -> Result<CreateLensResponse, LensManagerServiceClientError> {
        let raw_request: CreateLensRequestProto = request.into();
        let raw_response = self
            .inner
            .create_lens(raw_request)
            .await
            .map_err(|s| LensManagerServiceClientError::Status(s.into()))?;
        let proto_response = raw_response.into_inner();
        let response = proto_response.try_into()?;
        Ok(response)
    }

    /// MergeLens adds the scope of one lens to another
    pub async fn merge_lens(
        &mut self,
        request: MergeLensRequest,
    ) -> Result<MergeLensResponse, LensManagerServiceClientError> {
        let raw_request: MergeLensRequestProto = request.into();
        let raw_response = self
            .inner
            .merge_lens(raw_request)
            .await
            .map_err(|s| LensManagerServiceClientError::Status(s.into()))?;
        let proto_response = raw_response.into_inner();
        let response = proto_response.try_into()?;
        Ok(response)
    }

    /// CloseLens will remove a Lens node from the graph, detaching it from its scope
    pub async fn close_lens(
        &mut self,
        request: CloseLensRequest,
    ) -> Result<CloseLensResponse, LensManagerServiceClientError> {
        let raw_request: CloseLensRequestProto = request.into();
        let raw_response = self
            .inner
            .close_lens(raw_request)
            .await
            .map_err(|s| LensManagerServiceClientError::Status(s.into()))?;
        let proto_response = raw_response.into_inner();
        let response = proto_response.try_into()?;
        Ok(response)
    }

    /// Adds a given entity node to the scope of a lens
    pub async fn add_node_to_scope(
        &mut self,
        request: AddNodeToScopeRequest,
    ) -> Result<AddNodeToScopeResponse, LensManagerServiceClientError> {
        let raw_request: AddNodeToScopeRequestProto = request.into();
        let raw_response = self
            .inner
            .add_node_to_scope(raw_request)
            .await
            .map_err(|s| LensManagerServiceClientError::Status(s.into()))?;
        let proto_response = raw_response.into_inner();
        let response = proto_response.try_into()?;
        Ok(response)
    }

    /// Remove a node from a given lens's scope
    pub async fn remove_node_from_scope(
        &mut self,
        request: RemoveNodeFromScopeRequest,
    ) -> Result<RemoveNodeFromScopeResponse, LensManagerServiceClientError> {
        let raw_request: RemoveNodeFromScopeRequestProto = request.into();
        let raw_response = self
            .inner
            .remove_node_from_scope(raw_request)
            .await
            .map_err(|s| LensManagerServiceClientError::Status(s.into()))?;
        let proto_response = raw_response.into_inner();
        let response = proto_response.try_into()?;
        Ok(response)
    }

    /// Remove a node from all of the lens scopes it is attached to
    pub async fn remove_node_from_all_scopes(
        &mut self,
        request: RemoveNodeFromAllScopesRequest,
    ) -> Result<RemoveNodeFromAllScopesResponse, LensManagerServiceClientError> {
        let raw_request: RemoveNodeFromAllScopesRequestProto = request.into();
        let raw_response = self
            .inner
            .remove_node_from_all_scopes(raw_request)
            .await
            .map_err(|s| LensManagerServiceClientError::Status(s.into()))?;
        let proto_response = raw_response.into_inner();
        let response = proto_response.try_into()?;
        Ok(response)
    }
}
