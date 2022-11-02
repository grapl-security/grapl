use crate::{
    protobufs::graplinc::grapl::api::scylla_provisioner::v1beta1::{
        ProvisionGraphForTenantRequest as ProvisionGraphForTenantRequestProto,
        ProvisionGraphForTenantResponse as ProvisionGraphForTenantResponseProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

#[derive(Debug, Clone)]
pub struct ProvisionGraphForTenantRequest {
    pub tenant_id: uuid::Uuid,
}

impl TryFrom<ProvisionGraphForTenantRequestProto> for ProvisionGraphForTenantRequest {
    type Error = SerDeError;

    fn try_from(request: ProvisionGraphForTenantRequestProto) -> Result<Self, Self::Error> {
        Ok(Self {
            tenant_id: request
                .tenant_id
                .ok_or_else(|| SerDeError::MissingField("tenant_id"))?
                .into(),
        })
    }
}

impl From<ProvisionGraphForTenantRequest> for ProvisionGraphForTenantRequestProto {
    fn from(request: ProvisionGraphForTenantRequest) -> Self {
        Self {
            tenant_id: Some(request.tenant_id.into()),
        }
    }
}

impl type_url::TypeUrl for ProvisionGraphForTenantRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.scylla_provisioner.v1beta1.ProvisionGraphForTenantRequest";
}

impl serde_impl::ProtobufSerializable for ProvisionGraphForTenantRequest {
    type ProtobufMessage = ProvisionGraphForTenantRequestProto;
}

#[derive(Debug, Clone)]
pub struct ProvisionGraphForTenantResponse {}

impl TryFrom<ProvisionGraphForTenantResponseProto> for ProvisionGraphForTenantResponse {
    type Error = SerDeError;

    fn try_from(request: ProvisionGraphForTenantResponseProto) -> Result<Self, Self::Error> {
        let ProvisionGraphForTenantResponseProto {} = request;
        Ok(Self {})
    }
}

impl From<ProvisionGraphForTenantResponse> for ProvisionGraphForTenantResponseProto {
    fn from(request: ProvisionGraphForTenantResponse) -> Self {
        let ProvisionGraphForTenantResponse {} = request;
        Self {}
    }
}

impl type_url::TypeUrl for ProvisionGraphForTenantResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.scylla_provisioner.v1beta1.ProvisionGraphForTenantResponse";
}

impl serde_impl::ProtobufSerializable for ProvisionGraphForTenantResponse {
    type ProtobufMessage = ProvisionGraphForTenantResponseProto;
}
