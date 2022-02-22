// use org_management::orgmanagement_client::OrgClient;
// use org_management::OrgRequest;

use rust_proto::org_management::{
    org_management_service_client::OrgManagementServiceClient as _OrgManagementServiceClient,
    CreateOrgRequest,
    CreateOrgResponse,
    CreateUserRequest,
    CreateUserResponse,
    CreateOrgRequestProto,
    CreateUserRequestProto,
};


use tonic::{
    codegen::{
        Body,
        StdError,
    },
    Status,
};

#[derive(Debug, thiserror::Error)]
pub enum OrgManagementDeserializationError {
    #[error("Missing a required field")]
    MissingRequiredField(&'static str),
    #[error("Empty field")]
    EmptyField(&'static str),
}


#[derive(Debug, thiserror::Error)]
pub enum OrgManagementClientError {
    #[error("GrpcStatus {0}")]
    GrpcStatus(#[from] Status),
    #[error("DeserializeError {0}")]
    DeserializeError(#[from] OrgManagementDeserializationError),
}

#[derive(Debug)]
pub struct OrgManagementServiceClient<T>{
    inner: _OrgManagementServiceClient<T>,
}


impl<T> OrgManagementServiceClient<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::ResponseBody: Body + Send + 'static,
    T::Error: Into<StdError>,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub async fn create_org(
        &mut self,
        request: CreateOrgRequest,
    ) -> Result<CreateOrgResponse, OrgManagementClientError> {
        let response = self
            .inner
            .create_org(CreateOrgRequestProto::from(request))
            .await?;
        Ok(response.into_inner().try_into()?)

    }

    pub async fn create_user(
        &mut self,
        request: CreateUserRequest,
    ) -> Result<CreateUserResponse, OrgManagementClientError> {
        let response = self
            .inner
            .create_user(CreateUserRequestProto::from(request))
            .await?;
        Ok(response.into_inner().try_into()?)
    }


}



// async fn grpc_client() -> Result<(), Box<dyn std::error::Error>> {
//     let mut client = OrgClient::connect("http://[::1]:50051").await?;
//
//     let request = tonic::Request::new(OrgRequest {
//         name: "Tonic".into(),
//     });
//
//     let response = client.create_org(request).await?;
//
//     println!("RESPONSE={:?}", response);
//
//     Ok(())
// }
