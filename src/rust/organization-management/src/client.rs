use rust_proto::organization_management::{
    organization_management_service_client::OrganizationManagementServiceClient as _OrganizationManagementServiceClient,
    CreateOrganizationRequest,
    CreateOrganizationResponse,
    CreateUserRequest,
    CreateUserResponse,
    CreateOrganizationRequestProto,
    CreateUserRequestProto,
    OrganizationManagementDeserializationError
};

use tonic::{
    codegen::{
        Body,
        StdError,
    },
    Status,
};

use std::convert::{
    TryInto,
};


#[derive(Debug, thiserror::Error)]
pub enum OrganizationManagementServiceClientError {
    #[error("GrpcStatus {0}")]
    GrpcStatus(#[from] Status),
    #[error("DeserializeError {0}")]
    DeserializeError(#[from] OrganizationManagementDeserializationError),
}

#[derive(Debug)]
pub struct OrganizationManagementServiceClient<T>{
    inner: _OrganizationManagementServiceClient<T>,
}

impl<T> OrganizationManagementServiceClient<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::ResponseBody: Body + Send + 'static,
    T::Error: Into<StdError>,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub fn new(inner: _OrganizationManagementServiceClient<T>) -> Self {
        Self { inner }
    }

    /// Create a new organization
    pub async fn create_organization(
        &mut self,
        request: CreateOrganizationRequest,
    ) -> Result<CreateOrganizationResponse, OrganizationManagementServiceClientError> {
        let response = self
            .inner
            .create_organization(CreateOrganizationRequestProto::from(request))
            .await?;
        Ok(response.into_inner().try_into()?)

    }

    /// Creates a new user
    pub async fn create_user(
        &mut self,
        request: CreateUserRequest,
    ) -> Result<CreateUserResponse, OrganizationManagementServiceClientError> {
        let response = self
            .inner
            .create_user(CreateUserRequestProto::from(request))
            .await?;
        Ok(response.into_inner().try_into()?)
    }
}
