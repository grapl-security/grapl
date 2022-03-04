use std::{
    convert::TryInto,
    time::Duration,
};

use rust_proto::organization_management::{
    organization_management_service_client::OrganizationManagementServiceClient as _OrganizationManagementServiceClient,
    CreateOrganizationRequest,
    CreateOrganizationRequestProto,
    CreateOrganizationResponse,
    CreateUserRequest,
    CreateUserRequestProto,
    CreateUserResponse,
    OrganizationManagementDeserializationError,
};
use tonic::{
    codegen::{
        Body,
        StdError,
    },
    transport::Endpoint,
    Status,
};

const ADDRESS_ENV_VAR: &'static str = "ORGANIZATION_MANAGEMENT_ADDRESS";

#[derive(Debug, thiserror::Error)]
pub enum OrganizationManagementServiceClientError {
    #[error("GrpcStatus {0}")]
    GrpcStatus(#[from] Status),
    #[error("DeserializeError {0}")]
    DeserializeError(#[from] OrganizationManagementDeserializationError),
}

#[derive(Debug)]
pub struct OrganizationManagementServiceClient<T> {
    inner: _OrganizationManagementServiceClient<T>,
}

impl OrganizationManagementServiceClient<tonic::transport::Channel> {
    /// Create a client from environment
    #[tracing::instrument(err)]
    pub async fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let address = std::env::var(ADDRESS_ENV_VAR).expect(ADDRESS_ENV_VAR);
        Self::from_endpoint(address).await
    }

    /// Create a client from a specific endpoint
    #[tracing::instrument(err)]
    pub async fn from_endpoint(address: String) -> Result<Self, Box<dyn std::error::Error>> {
        tracing::debug!(message = "Connecting to organization management endpoint");

        // TODO: It might make sense to make these values configurable.
        let endpoint = Endpoint::from_shared(address)?
            .timeout(Duration::from_secs(5))
            .concurrency_limit(30);
        let channel = endpoint.connect().await?;
        Ok(Self::new(_OrganizationManagementServiceClient::new(
            channel,
        )))
    }
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
