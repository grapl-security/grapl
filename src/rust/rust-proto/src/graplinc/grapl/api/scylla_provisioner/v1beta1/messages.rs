use crate::{
    protobufs::graplinc::grapl::api::scylla_provisioner::v1beta1::{
        DeployGraphSchemasRequest as DeployGraphSchemasRequestProto,
        DeployGraphSchemasResponse as DeployGraphSchemasResponseProto,
    },
    SerDeError,
};

#[derive(Debug, Clone)]
pub struct DeployGraphSchemasRequest {
    pub tenant_id: uuid::Uuid,
}

impl TryFrom<DeployGraphSchemasRequestProto> for DeployGraphSchemasRequest {
    type Error = SerDeError;

    fn try_from(request: DeployGraphSchemasRequestProto) -> Result<Self, Self::Error> {
        Ok(Self {
            tenant_id: request
                .tenant_id
                .ok_or_else(|| SerDeError::MissingField("tenant_id"))?
                .into(),
        })
    }
}

impl From<DeployGraphSchemasRequest> for DeployGraphSchemasRequestProto {
    fn from(request: DeployGraphSchemasRequest) -> Self {
        Self {
            tenant_id: Some(request.tenant_id.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeployGraphSchemasResponse {}

impl TryFrom<DeployGraphSchemasResponseProto> for DeployGraphSchemasResponse {
    type Error = SerDeError;

    fn try_from(request: DeployGraphSchemasResponseProto) -> Result<Self, Self::Error> {
        let DeployGraphSchemasResponseProto {} = request;
        Ok(Self {})
    }
}

impl From<DeployGraphSchemasResponse> for DeployGraphSchemasResponseProto {
    fn from(request: DeployGraphSchemasResponse) -> Self {
        let DeployGraphSchemasResponse {} = request;
        Self {}
    }
}
