use std::fmt::Debug;

use proto::{
    create_plugin_request_v2,
    create_plugin_response_v2,
};

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

impl ProtobufSerializable for Plugin {
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

pub enum CreatePluginRequestV2 {
    Metadata(CreatePluginRequestMetadata),
    Chunk(CreatePluginRequestChunk),
}

impl type_url::TypeUrl for CreatePluginRequestV2 {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.CreatePluginRequestV2";
}

impl TryFrom<proto::CreatePluginRequestV2> for CreatePluginRequestV2 {
    type Error = SerDeError;

    fn try_from(value: proto::CreatePluginRequestV2) -> Result<Self, Self::Error> {
        match value.inner {
            Some(create_plugin_request_v2::Inner::Metadata(m)) => {
                Ok(CreatePluginRequestV2::Metadata(m.try_into()?))
            }
            Some(create_plugin_request_v2::Inner::Chunk(c)) => {
                Ok(CreatePluginRequestV2::Chunk(c.try_into()?))
            }
            _ => Err(SerDeError::UnknownVariant("CreatePluginRequestV2.inner")),
        }
    }
}

impl From<CreatePluginRequestV2> for proto::CreatePluginRequestV2 {
    fn from(value: CreatePluginRequestV2) -> Self {
        proto::CreatePluginRequestV2 {
            inner: Some(match value {
                CreatePluginRequestV2::Metadata(m) => {
                    create_plugin_request_v2::Inner::Metadata(m.into())
                }
                CreatePluginRequestV2::Chunk(c) => create_plugin_request_v2::Inner::Chunk(c.into()),
            }),
        }
    }
}

impl ProtobufSerializable for CreatePluginRequestV2 {
    type ProtobufMessage = proto::CreatePluginRequestV2;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatePluginRequestMetadata {
    /// Tenant that is deploying this plugin
    pub tenant_id: uuid::Uuid,
    /// The string value to display to a user, non-empty
    pub display_name: String,
    /// The type of the plugin
    pub plugin_type: PluginType,
}

impl type_url::TypeUrl for CreatePluginRequestMetadata {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.CreatePluginRequestMetadata";
}

impl TryFrom<proto::CreatePluginRequestMetadata> for CreatePluginRequestMetadata {
    type Error = SerDeError;

    fn try_from(value: proto::CreatePluginRequestMetadata) -> Result<Self, Self::Error> {
        let plugin_type = value.plugin_type().try_into()?;

        let tenant_id = value
            .tenant_id
            .ok_or(SerDeError::MissingField("CreatePluginRequest.tenant_id"))?
            .into();
        let display_name = value.display_name;

        if display_name.is_empty() {
            return Err(SerDeError::MissingField("CreatePluginRequest.display_name"));
        }

        Ok(Self {
            tenant_id,
            display_name,
            plugin_type,
        })
    }
}

impl From<CreatePluginRequestMetadata> for proto::CreatePluginRequestMetadata {
    fn from(value: CreatePluginRequestMetadata) -> Self {
        let plugin_type: proto::PluginType = value.plugin_type.into();
        Self {
            tenant_id: Some(value.tenant_id.into()),
            display_name: value.display_name,
            plugin_type: plugin_type as i32,
        }
    }
}

impl ProtobufSerializable for CreatePluginRequestMetadata {
    type ProtobufMessage = proto::CreatePluginRequestMetadata;
}

/////////////

#[derive(Debug, Clone, PartialEq)]
pub struct CreatePluginRequestChunk {
    /// the actual plugin code
    pub plugin_artifact: Vec<u8>,
}

impl type_url::TypeUrl for CreatePluginRequestChunk {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.CreatePluginRequestChunk";
}

impl TryFrom<proto::CreatePluginRequestChunk> for CreatePluginRequestChunk {
    type Error = SerDeError;

    fn try_from(value: proto::CreatePluginRequestChunk) -> Result<Self, Self::Error> {
        let plugin_artifact = value.plugin_artifact;
        if plugin_artifact.is_empty() {
            return Err(SerDeError::MissingField(
                "CreatePluginRequestChunk.plugin_artifact",
            ));
        }
        Ok(Self { plugin_artifact })
    }
}

impl From<CreatePluginRequestChunk> for proto::CreatePluginRequestChunk {
    fn from(value: CreatePluginRequestChunk) -> Self {
        Self {
            plugin_artifact: value.plugin_artifact,
        }
    }
}

impl ProtobufSerializable for CreatePluginRequestChunk {
    type ProtobufMessage = proto::CreatePluginRequestChunk;
}

pub enum CreatePluginResponseV2 {
    AwaitingChunk,
    PluginId(uuid::Uuid),
}

impl type_url::TypeUrl for CreatePluginResponseV2 {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.CreatePluginResponseV2";
}

impl TryFrom<proto::CreatePluginResponseV2> for CreatePluginResponseV2 {
    type Error = SerDeError;

    fn try_from(value: proto::CreatePluginResponseV2) -> Result<Self, Self::Error> {
        match value.inner {
            Some(create_plugin_response_v2::Inner::AwaitingChunk(_)) => {
                Ok(CreatePluginResponseV2::AwaitingChunk)
            }
            Some(create_plugin_response_v2::Inner::PluginId(p)) => {
                Ok(CreatePluginResponseV2::PluginId(p.into()))
            }
            _ => Err(SerDeError::UnknownVariant("CreatePluginResponseV2.inner")),
        }
    }
}

impl From<CreatePluginResponseV2> for proto::CreatePluginResponseV2 {
    fn from(value: CreatePluginResponseV2) -> Self {
        proto::CreatePluginResponseV2 {
            inner: Some(match value {
                CreatePluginResponseV2::AwaitingChunk => {
                    create_plugin_response_v2::Inner::AwaitingChunk(
                        proto::CreatePluginResponseAwaitingChunk::default(),
                    )
                }
                CreatePluginResponseV2::PluginId(p) => {
                    create_plugin_response_v2::Inner::PluginId(p.into())
                }
            }),
        }
    }
}

impl ProtobufSerializable for CreatePluginResponseV2 {
    type ProtobufMessage = proto::CreatePluginResponseV2;
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

impl ProtobufSerializable for DeployPluginRequest {
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
impl ProtobufSerializable for DeployPluginResponse {
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
impl ProtobufSerializable for GetAnalyzersForTenantRequest {
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
impl ProtobufSerializable for GetAnalyzersForTenantResponse {
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
impl ProtobufSerializable for GetGeneratorsForEventSourceRequest {
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

impl ProtobufSerializable for GetGeneratorsForEventSourceResponse {
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
impl ProtobufSerializable for GetPluginRequest {
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
impl ProtobufSerializable for GetPluginResponse {
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
impl ProtobufSerializable for TearDownPluginRequest {
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

impl ProtobufSerializable for TearDownPluginResponse {
    type ProtobufMessage = proto::TearDownPluginResponse;
}
