use std::fmt::Debug;

pub use crate::graplinc::grapl::api::plugin_registry::{
    v1beta1_client::{
        PluginRegistryServiceClient,
        PluginRegistryServiceClientError,
    },
    v1beta1_server::{
        PluginRegistryApi,
        PluginRegistryServer,
    },
};
use crate::{
    protobufs::graplinc::grapl::api::plugin_registry::v1beta1 as proto,
    serde_impl::ProtobufSerializable,
    type_url,
    SerDeError,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PluginType {
    Generator,
    Analyzer,
}

impl PluginType {
    pub fn type_name(&self) -> &'static str {
        match self {
            PluginType::Generator => "generator",
            PluginType::Analyzer => "analyzer",
        }
    }
}

impl TryFrom<proto::PluginType> for PluginType {
    type Error = SerDeError;

    fn try_from(value: proto::PluginType) -> Result<Self, Self::Error> {
        match value {
            proto::PluginType::Unspecified => Err(SerDeError::UnknownVariant("PluginType")),
            proto::PluginType::Generator => Ok(PluginType::Generator),
            proto::PluginType::Analyzer => Ok(PluginType::Analyzer),
        }
        // todo!()
    }
}

impl From<PluginType> for proto::PluginType {
    fn from(value: PluginType) -> Self {
        match value {
            PluginType::Generator => proto::PluginType::Generator,
            PluginType::Analyzer => proto::PluginType::Analyzer,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plugin {
    /// unique identifier for this plugin
    pub plugin_id: uuid::Uuid,
    /// The string value to display to a user, non-empty
    pub display_name: String,
    /// The type of the plugin
    pub plugin_type: PluginType,
    /// The byte representation of the plugin executable
    pub plugin_binary: Vec<u8>,
}

impl ProtobufSerializable<Plugin> for Plugin {
    type ProtobufMessage = proto::Plugin;
}

impl type_url::TypeUrl for Plugin {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.Plugin";
}

impl TryFrom<proto::Plugin> for Plugin {
    type Error = SerDeError;

    fn try_from(value: proto::Plugin) -> Result<Self, Self::Error> {
        let plugin_type = value.plugin_type().try_into()?;
        let display_name = value.display_name;
        let plugin_binary = value.plugin_binary;
        let plugin_id = value
            .plugin_id
            .ok_or(SerDeError::MissingField("Plugin.plugin_id"))?
            .into();

        Ok(Self {
            plugin_id,
            display_name,
            plugin_type,
            plugin_binary,
        })
    }
}

impl From<Plugin> for proto::Plugin {
    fn from(value: Plugin) -> Self {
        let plugin_type: proto::PluginType = value.plugin_type.into();
        Self {
            plugin_id: Some(value.plugin_id.into()),
            display_name: value.display_name,
            plugin_type: plugin_type as i32,
            plugin_binary: value.plugin_binary,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatePluginRequest {
    /// the actual plugin code
    pub plugin_artifact: Vec<u8>,
    /// Tenant that is deploying this plugin
    pub tenant_id: uuid::Uuid,
    /// The string value to display to a user, non-empty
    pub display_name: String,
    /// The type of the plugin
    pub plugin_type: PluginType,
}

impl type_url::TypeUrl for CreatePluginRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.CreatePluginRequest";
}

impl TryFrom<proto::CreatePluginRequest> for CreatePluginRequest {
    type Error = SerDeError;

    fn try_from(value: proto::CreatePluginRequest) -> Result<Self, Self::Error> {
        let plugin_type = value.plugin_type().try_into()?;

        let tenant_id = value
            .tenant_id
            .ok_or(SerDeError::MissingField("CreatePluginRequest.tenant_id"))?
            .into();
        let display_name = value.display_name;
        let plugin_artifact = value.plugin_artifact;

        if display_name.is_empty() {
            return Err(SerDeError::MissingField("CreatePluginRequest.display_name"));
        }

        if plugin_artifact.is_empty() {
            return Err(SerDeError::MissingField(
                "CreatePluginRequest.plugin_artifact",
            ));
        }

        Ok(Self {
            plugin_artifact,
            tenant_id,
            display_name,
            plugin_type,
        })
    }
}

impl From<CreatePluginRequest> for proto::CreatePluginRequest {
    fn from(value: CreatePluginRequest) -> Self {
        let plugin_type: proto::PluginType = value.plugin_type.into();
        Self {
            plugin_artifact: value.plugin_artifact,
            tenant_id: Some(value.tenant_id.into()),
            display_name: value.display_name,
            plugin_type: plugin_type as i32,
        }
    }
}

impl ProtobufSerializable<CreatePluginRequest> for CreatePluginRequest {
    type ProtobufMessage = proto::CreatePluginRequest;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatePluginResponse {
    /// The identity of the plugin that was created
    pub plugin_id: uuid::Uuid,
}

impl type_url::TypeUrl for CreatePluginResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.CreatePluginResponse";
}

impl TryFrom<proto::CreatePluginResponse> for CreatePluginResponse {
    type Error = SerDeError;

    fn try_from(value: proto::CreatePluginResponse) -> Result<Self, Self::Error> {
        let plugin_id = value
            .plugin_id
            .ok_or(SerDeError::MissingField("CreatePluginResponse.plugin_id"))?
            .into();

        Ok(Self { plugin_id })
    }
}

impl From<CreatePluginResponse> for proto::CreatePluginResponse {
    fn from(value: CreatePluginResponse) -> Self {
        Self {
            plugin_id: Some(value.plugin_id.into()),
        }
    }
}

impl ProtobufSerializable<CreatePluginResponse> for CreatePluginResponse {
    type ProtobufMessage = proto::CreatePluginResponse;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeployPluginRequest {
    pub plugin_id: uuid::Uuid,
}

impl type_url::TypeUrl for DeployPluginRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.DeployPluginRequest";
}

impl TryFrom<proto::DeployPluginRequest> for DeployPluginRequest {
    type Error = SerDeError;

    fn try_from(value: proto::DeployPluginRequest) -> Result<Self, Self::Error> {
        let plugin_id = value
            .plugin_id
            .ok_or(SerDeError::MissingField("DeployPluginRequest.plugin_id"))?
            .into();

        Ok(Self { plugin_id })
    }
}

impl From<DeployPluginRequest> for proto::DeployPluginRequest {
    fn from(value: DeployPluginRequest) -> Self {
        Self {
            plugin_id: Some(value.plugin_id.into()),
        }
    }
}

impl ProtobufSerializable<DeployPluginRequest> for DeployPluginRequest {
    type ProtobufMessage = proto::DeployPluginRequest;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeployPluginResponse {}

impl type_url::TypeUrl for DeployPluginResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.DeployPluginResponse";
}

impl TryFrom<proto::DeployPluginResponse> for DeployPluginResponse {
    type Error = SerDeError;

    fn try_from(_value: proto::DeployPluginResponse) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<DeployPluginResponse> for proto::DeployPluginResponse {
    fn from(_value: DeployPluginResponse) -> Self {
        Self {}
    }
}
impl ProtobufSerializable<DeployPluginResponse> for DeployPluginResponse {
    type ProtobufMessage = proto::DeployPluginResponse;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetAnalyzersForTenantRequest {
    /// The tenant id for the tenant whose analyzers we wish to fetch
    pub tenant_id: uuid::Uuid,
}

impl type_url::TypeUrl for GetAnalyzersForTenantRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.GetAnalyzersForTenantRequest";
}

impl TryFrom<proto::GetAnalyzersForTenantRequest> for GetAnalyzersForTenantRequest {
    type Error = SerDeError;

    fn try_from(value: proto::GetAnalyzersForTenantRequest) -> Result<Self, Self::Error> {
        let tenant_id = value
            .tenant_id
            .ok_or(SerDeError::MissingField(
                "GetAnalyzersForTenantRequest.tenant_id",
            ))?
            .into();

        Ok(Self { tenant_id })
    }
}

impl From<GetAnalyzersForTenantRequest> for proto::GetAnalyzersForTenantRequest {
    fn from(value: GetAnalyzersForTenantRequest) -> Self {
        Self {
            tenant_id: Some(value.tenant_id.into()),
        }
    }
}
impl ProtobufSerializable<GetAnalyzersForTenantRequest> for GetAnalyzersForTenantRequest {
    type ProtobufMessage = proto::GetAnalyzersForTenantRequest;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetAnalyzersForTenantResponse {
    /// The plugin ids for the analyzers belonging to a tenant
    pub plugin_ids: Vec<uuid::Uuid>,
}

impl type_url::TypeUrl for GetAnalyzersForTenantResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.GetAnalyzersForTenantResponse";
}

impl TryFrom<proto::GetAnalyzersForTenantResponse> for GetAnalyzersForTenantResponse {
    type Error = SerDeError;

    fn try_from(value: proto::GetAnalyzersForTenantResponse) -> Result<Self, Self::Error> {
        if value.plugin_ids.is_empty() {
            return Err(SerDeError::MissingField(
                "GetAnalyzersForTenantResponse.plugin_ids",
            ));
        }
        let plugin_ids = value.plugin_ids.into_iter().map(uuid::Uuid::from).collect();

        Ok(Self { plugin_ids })
    }
}

impl From<GetAnalyzersForTenantResponse> for proto::GetAnalyzersForTenantResponse {
    fn from(value: GetAnalyzersForTenantResponse) -> Self {
        Self {
            plugin_ids: value.plugin_ids.into_iter().map(uuid::Uuid::into).collect(),
        }
    }
}
impl ProtobufSerializable<GetAnalyzersForTenantResponse> for GetAnalyzersForTenantResponse {
    type ProtobufMessage = proto::GetAnalyzersForTenantResponse;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetGeneratorsForEventSourceRequest {
    /// The event source id
    pub event_source_id: uuid::Uuid,
}

impl type_url::TypeUrl for GetGeneratorsForEventSourceRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.GetGeneratorsForEventSourceRequest";
}

impl TryFrom<proto::GetGeneratorsForEventSourceRequest> for GetGeneratorsForEventSourceRequest {
    type Error = SerDeError;

    fn try_from(value: proto::GetGeneratorsForEventSourceRequest) -> Result<Self, Self::Error> {
        let event_source_id = value
            .event_source_id
            .ok_or(SerDeError::MissingField(
                "GetGeneratorsForEventSourceRequest.event_source_id",
            ))?
            .into();

        Ok(Self { event_source_id })
    }
}

impl From<GetGeneratorsForEventSourceRequest> for proto::GetGeneratorsForEventSourceRequest {
    fn from(value: GetGeneratorsForEventSourceRequest) -> Self {
        Self {
            event_source_id: Some(value.event_source_id.into()),
        }
    }
}
impl ProtobufSerializable<GetGeneratorsForEventSourceRequest>
    for GetGeneratorsForEventSourceRequest
{
    type ProtobufMessage = proto::GetGeneratorsForEventSourceRequest;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetGeneratorsForEventSourceResponse {
    pub plugin_ids: Vec<uuid::Uuid>,
}

impl type_url::TypeUrl for GetGeneratorsForEventSourceResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.GetGeneratorsForEventSourceResponse";
}

impl TryFrom<proto::GetGeneratorsForEventSourceResponse> for GetGeneratorsForEventSourceResponse {
    type Error = SerDeError;

    fn try_from(value: proto::GetGeneratorsForEventSourceResponse) -> Result<Self, Self::Error> {
        if value.plugin_ids.is_empty() {
            return Err(SerDeError::MissingField(
                "GetGeneratorsForEventSourceResponse.plugin_ids",
            ));
        }
        let plugin_ids = value.plugin_ids.into_iter().map(uuid::Uuid::from).collect();

        Ok(Self { plugin_ids })
    }
}

impl From<GetGeneratorsForEventSourceResponse> for proto::GetGeneratorsForEventSourceResponse {
    fn from(value: GetGeneratorsForEventSourceResponse) -> Self {
        Self {
            plugin_ids: value.plugin_ids.into_iter().map(uuid::Uuid::into).collect(),
        }
    }
}

impl ProtobufSerializable<GetGeneratorsForEventSourceResponse>
    for GetGeneratorsForEventSourceResponse
{
    type ProtobufMessage = proto::GetGeneratorsForEventSourceResponse;
}
#[derive(Debug, Clone, PartialEq)]
pub struct GetPluginRequest {
    /// The identity of the plugin
    pub plugin_id: uuid::Uuid,
    /// The tenant for which the plugin belongs to
    pub tenant_id: uuid::Uuid,
}

impl type_url::TypeUrl for GetPluginRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.GetPluginRequest";
}

impl TryFrom<proto::GetPluginRequest> for GetPluginRequest {
    type Error = SerDeError;

    fn try_from(value: proto::GetPluginRequest) -> Result<Self, Self::Error> {
        let plugin_id = value
            .plugin_id
            .ok_or(SerDeError::MissingField("GetPluginRequest.plugin_id"))?
            .into();

        let tenant_id = value
            .tenant_id
            .ok_or(SerDeError::MissingField(
                "GetAnalyzersForTenantRequest.tenant_id",
            ))?
            .into();

        Ok(Self {
            plugin_id,
            tenant_id,
        })
    }
}

impl From<GetPluginRequest> for proto::GetPluginRequest {
    fn from(value: GetPluginRequest) -> Self {
        Self {
            plugin_id: Some(value.plugin_id.into()),
            tenant_id: Some(value.tenant_id.into()),
        }
    }
}
impl ProtobufSerializable<GetPluginRequest> for GetPluginRequest {
    type ProtobufMessage = proto::GetPluginRequest;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetPluginResponse {
    pub plugin: Plugin,
}

impl type_url::TypeUrl for GetPluginResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.GetPluginResponse";
}

impl TryFrom<proto::GetPluginResponse> for GetPluginResponse {
    type Error = SerDeError;

    fn try_from(value: proto::GetPluginResponse) -> Result<Self, Self::Error> {
        let plugin = value
            .plugin
            .ok_or(SerDeError::MissingField("GetPluginResponse.plugin"))?
            .try_into()?;

        Ok(Self { plugin })
    }
}

impl From<GetPluginResponse> for proto::GetPluginResponse {
    fn from(value: GetPluginResponse) -> Self {
        Self {
            plugin: Some(value.plugin.into()),
        }
    }
}
impl ProtobufSerializable<GetPluginResponse> for GetPluginResponse {
    type ProtobufMessage = proto::GetPluginResponse;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TearDownPluginRequest {
    pub plugin_id: uuid::Uuid,
}

impl type_url::TypeUrl for TearDownPluginRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.TearDownPluginRequest";
}

impl TryFrom<proto::TearDownPluginRequest> for TearDownPluginRequest {
    type Error = SerDeError;

    fn try_from(value: proto::TearDownPluginRequest) -> Result<Self, Self::Error> {
        let plugin_id = value
            .plugin_id
            .ok_or(SerDeError::MissingField("TearDownPluginRequest.plugin_id"))?
            .into();

        Ok(Self { plugin_id })
    }
}

impl From<TearDownPluginRequest> for proto::TearDownPluginRequest {
    fn from(value: TearDownPluginRequest) -> Self {
        Self {
            plugin_id: Some(value.plugin_id.into()),
        }
    }
}
impl ProtobufSerializable<TearDownPluginRequest> for TearDownPluginRequest {
    type ProtobufMessage = proto::TearDownPluginRequest;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TearDownPluginResponse {}

impl type_url::TypeUrl for TearDownPluginResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.TearDownPluginResponse";
}

impl TryFrom<proto::TearDownPluginResponse> for TearDownPluginResponse {
    type Error = SerDeError;

    fn try_from(_value: proto::TearDownPluginResponse) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<TearDownPluginResponse> for proto::TearDownPluginResponse {
    fn from(_: TearDownPluginResponse) -> Self {
        Self {}
    }
}

impl ProtobufSerializable<TearDownPluginResponse> for TearDownPluginResponse {
    type ProtobufMessage = proto::TearDownPluginResponse;
}
