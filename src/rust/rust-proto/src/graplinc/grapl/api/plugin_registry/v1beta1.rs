use std::fmt::Debug;

use bytes::Bytes;
use proto::create_plugin_request;

pub use crate::graplinc::grapl::api::plugin_registry::{
    v1beta1_client::PluginRegistryServiceClient,
    v1beta1_server::{
        PluginRegistryApi,
        PluginRegistryServer,
    },
};
use crate::{
    protobufs::graplinc::grapl::api::plugin_registry::{
        v1beta1 as proto,
        v1beta1::get_plugin_health_response::PluginHealthStatus as PluginHealthStatusProto,
    },
    serde_impl::ProtobufSerializable,
    type_url,
    SerDeError,
};

//
// PluginType
//

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

impl TryFrom<&str> for PluginType {
    type Error = SerDeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "generator" => Ok(PluginType::Generator),
            "analyzer" => Ok(PluginType::Analyzer),
            _ => Err(SerDeError::UnknownVariant("PluginType")),
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

//
// PluginMetadata
//

#[derive(Debug, Clone, PartialEq)]
pub struct PluginMetadata {
    /// The platform tenant this plugin belongs to
    tenant_id: uuid::Uuid,
    /// The string value to display to a user, non-empty
    display_name: String,
    /// The type of the plugin
    plugin_type: PluginType,
    /// The event source id associated with this plugin. Present if
    /// PluginType::Generator, absent otherwise.
    event_source_id: Option<uuid::Uuid>,
}

impl PluginMetadata {
    pub fn new(
        tenant_id: uuid::Uuid,
        display_name: String,
        plugin_type: PluginType,
        event_source_id: Option<uuid::Uuid>,
    ) -> Self {
        Self {
            tenant_id,
            display_name,
            plugin_type,
            event_source_id,
        }
    }

    pub fn tenant_id(&self) -> uuid::Uuid {
        self.tenant_id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn plugin_type(&self) -> PluginType {
        self.plugin_type
    }

    pub fn event_source_id(&self) -> Option<uuid::Uuid> {
        self.event_source_id
    }
}

impl type_url::TypeUrl for PluginMetadata {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.PluginMetadata";
}

impl TryFrom<proto::PluginMetadata> for PluginMetadata {
    type Error = SerDeError;

    fn try_from(value: proto::PluginMetadata) -> Result<Self, Self::Error> {
        let plugin_type = value.plugin_type().try_into()?;

        let event_source_id = match plugin_type {
            PluginType::Generator => {
                if let Some(event_source_id) = value.event_source_id {
                    Ok(Some(event_source_id.into()))
                } else {
                    Err(SerDeError::MissingField("event_source_id"))
                }
            }
            _ => {
                if value.event_source_id.is_some() {
                    Err(SerDeError::InvalidField {
                        field_name: "event_source_id",
                        assertion: "must be absent when plugin_type is not PluginType::Generator"
                            .to_string(),
                    })
                } else {
                    Ok(None)
                }
            }
        }?;

        let tenant_id = value
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?
            .into();

        let display_name = value.display_name;

        if display_name.is_empty() {
            return Err(SerDeError::MissingField("display_name"));
        }

        Ok(Self {
            tenant_id,
            display_name,
            plugin_type,
            event_source_id,
        })
    }
}

impl From<PluginMetadata> for proto::PluginMetadata {
    fn from(value: PluginMetadata) -> Self {
        let plugin_type: proto::PluginType = value.plugin_type.into();
        Self {
            tenant_id: Some(value.tenant_id.into()),
            display_name: value.display_name,
            plugin_type: plugin_type as i32,
            event_source_id: value.event_source_id.map(|id| id.into()),
        }
    }
}

impl ProtobufSerializable for PluginMetadata {
    type ProtobufMessage = proto::PluginMetadata;
}

//
// CreatePluginRequest
//

#[derive(Debug, Clone, PartialEq)]
pub enum CreatePluginRequest {
    Metadata(PluginMetadata),
    Chunk(Bytes),
}

impl type_url::TypeUrl for CreatePluginRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.CreatePluginRequest";
}

impl TryFrom<proto::CreatePluginRequest> for CreatePluginRequest {
    type Error = SerDeError;

    fn try_from(value: proto::CreatePluginRequest) -> Result<Self, Self::Error> {
        match value.inner {
            Some(create_plugin_request::Inner::Metadata(m)) => {
                Ok(CreatePluginRequest::Metadata(m.try_into()?))
            }
            Some(create_plugin_request::Inner::Chunk(c)) => Ok(CreatePluginRequest::Chunk(c)),
            _ => Err(SerDeError::UnknownVariant("CreatePluginRequest.inner")),
        }
    }
}

impl From<CreatePluginRequest> for proto::CreatePluginRequest {
    fn from(value: CreatePluginRequest) -> Self {
        proto::CreatePluginRequest {
            inner: Some(match value {
                CreatePluginRequest::Metadata(m) => {
                    create_plugin_request::Inner::Metadata(m.into())
                }
                CreatePluginRequest::Chunk(c) => create_plugin_request::Inner::Chunk(c),
            }),
        }
    }
}

impl ProtobufSerializable for CreatePluginRequest {
    type ProtobufMessage = proto::CreatePluginRequest;
}

//
// CreatePluginResponse
//

#[derive(Debug, Clone, PartialEq)]
pub struct CreatePluginResponse {
    /// The identity of the plugin that was created
    plugin_id: uuid::Uuid,
}

impl CreatePluginResponse {
    pub fn new(plugin_id: uuid::Uuid) -> Self {
        Self { plugin_id }
    }

    pub fn plugin_id(&self) -> uuid::Uuid {
        self.plugin_id
    }
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

impl ProtobufSerializable for CreatePluginResponse {
    type ProtobufMessage = proto::CreatePluginResponse;
}

//
// DeployPluginRequest
//

#[derive(Debug, Clone, PartialEq)]
pub struct DeployPluginRequest {
    plugin_id: uuid::Uuid,
}

impl DeployPluginRequest {
    pub fn new(plugin_id: uuid::Uuid) -> Self {
        Self { plugin_id }
    }

    pub fn plugin_id(&self) -> uuid::Uuid {
        self.plugin_id
    }
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

//
// DeployPluginResponse
//

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

//
// GetAnalyzersForTenantRequest
//

#[derive(Debug, Clone, PartialEq)]
pub struct GetAnalyzersForTenantRequest {
    /// The tenant id for the tenant whose analyzers we wish to fetch
    tenant_id: uuid::Uuid,
}

impl GetAnalyzersForTenantRequest {
    pub fn new(tenant_id: uuid::Uuid) -> Self {
        Self { tenant_id }
    }

    pub fn tenant_id(&self) -> uuid::Uuid {
        self.tenant_id
    }
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

//
// GetAnalyzersForTenantResponse
//

#[derive(Debug, Clone, PartialEq)]
pub struct GetAnalyzersForTenantResponse {
    /// The plugin ids for the analyzers belonging to a tenant
    plugin_ids: Vec<uuid::Uuid>,
}

impl GetAnalyzersForTenantResponse {
    pub fn new(plugin_ids: Vec<uuid::Uuid>) -> Self {
        Self { plugin_ids }
    }

    pub fn plugin_ids(&self) -> &[uuid::Uuid] {
        &self.plugin_ids
    }
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

//
// GetGeneratorsForEventSourceRequest
//

#[derive(Debug, Clone, PartialEq)]
pub struct GetGeneratorsForEventSourceRequest {
    /// The event source id
    event_source_id: uuid::Uuid,
}

impl GetGeneratorsForEventSourceRequest {
    pub fn new(event_source_id: uuid::Uuid) -> Self {
        Self { event_source_id }
    }

    pub fn event_source_id(&self) -> uuid::Uuid {
        self.event_source_id
    }
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

//
// GetGeneratorsForEventSourceResponse
//

#[derive(Debug, Clone, PartialEq)]
pub struct GetGeneratorsForEventSourceResponse {
    plugin_ids: Vec<uuid::Uuid>,
}

impl GetGeneratorsForEventSourceResponse {
    pub fn new(plugin_ids: Vec<uuid::Uuid>) -> Self {
        Self { plugin_ids }
    }

    pub fn plugin_ids(&self) -> &[uuid::Uuid] {
        &self.plugin_ids
    }
}

impl type_url::TypeUrl for GetGeneratorsForEventSourceResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.GetGeneratorsForEventSourceResponse";
}

impl From<proto::GetGeneratorsForEventSourceResponse> for GetGeneratorsForEventSourceResponse {
    fn from(value: proto::GetGeneratorsForEventSourceResponse) -> Self {
        let plugin_ids = value.plugin_ids.into_iter().map(uuid::Uuid::from).collect();

        Self { plugin_ids }
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

//
// GetPluginRequest
//

#[derive(Debug, Clone, PartialEq)]
pub struct GetPluginRequest {
    /// The identity of the plugin
    plugin_id: uuid::Uuid,
    /// The tenant for which the plugin belongs to
    tenant_id: uuid::Uuid,
}

impl GetPluginRequest {
    pub fn new(plugin_id: uuid::Uuid, tenant_id: uuid::Uuid) -> Self {
        Self {
            plugin_id,
            tenant_id,
        }
    }

    pub fn plugin_id(&self) -> uuid::Uuid {
        self.plugin_id
    }

    pub fn tenant_id(&self) -> uuid::Uuid {
        self.tenant_id
    }
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

//
// GetPluginResponse
//

#[derive(Debug, Clone, PartialEq)]
pub struct GetPluginResponse {
    plugin_id: uuid::Uuid,
    plugin_metadata: PluginMetadata,
}

impl GetPluginResponse {
    pub fn new(plugin_id: uuid::Uuid, plugin_metadata: PluginMetadata) -> Self {
        Self {
            plugin_id,
            plugin_metadata,
        }
    }

    pub fn plugin_id(&self) -> uuid::Uuid {
        self.plugin_id
    }

    pub fn plugin_metadata(&self) -> &PluginMetadata {
        &self.plugin_metadata
    }
}

impl type_url::TypeUrl for GetPluginResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.GetPluginResponse";
}

impl TryFrom<proto::GetPluginResponse> for GetPluginResponse {
    type Error = SerDeError;

    fn try_from(value: proto::GetPluginResponse) -> Result<Self, Self::Error> {
        let plugin_id = value
            .plugin_id
            .ok_or(SerDeError::MissingField("GetPluginResponse.plugin_id"))?
            .try_into()?;

        let plugin_metadata = value
            .plugin_metadata
            .ok_or(SerDeError::MissingField(
                "GetPluginResponse.plugin_metadata",
            ))?
            .try_into()?;

        Ok(Self {
            plugin_id,
            plugin_metadata,
        })
    }
}

impl From<GetPluginResponse> for proto::GetPluginResponse {
    fn from(value: GetPluginResponse) -> Self {
        Self {
            plugin_id: Some(value.plugin_id.into()),
            plugin_metadata: Some(value.plugin_metadata.into()),
        }
    }
}

impl ProtobufSerializable for GetPluginResponse {
    type ProtobufMessage = proto::GetPluginResponse;
}

//
// TearDownPluginRequest
//

#[derive(Debug, Clone, PartialEq)]
pub struct TearDownPluginRequest {
    plugin_id: uuid::Uuid,
}

impl TearDownPluginRequest {
    pub fn new(plugin_id: uuid::Uuid) -> Self {
        Self { plugin_id }
    }

    pub fn plugin_id(&self) -> uuid::Uuid {
        self.plugin_id
    }
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

//
// TearDownPluginResponse
//

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

//
// GetPluginHealthRequest
//

#[derive(Debug, Clone, PartialEq)]
pub struct GetPluginHealthRequest {
    plugin_id: uuid::Uuid,
}

impl GetPluginHealthRequest {
    pub fn new(plugin_id: uuid::Uuid) -> Self {
        Self { plugin_id }
    }

    pub fn plugin_id(&self) -> uuid::Uuid {
        self.plugin_id
    }
}

impl type_url::TypeUrl for GetPluginHealthRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.GetPluginHealthRequest";
}

impl TryFrom<proto::GetPluginHealthRequest> for GetPluginHealthRequest {
    type Error = SerDeError;

    fn try_from(value: proto::GetPluginHealthRequest) -> Result<Self, Self::Error> {
        let plugin_id = value
            .plugin_id
            .ok_or(SerDeError::MissingField("CreatePluginResponse.plugin_id"))?
            .into();
        Ok(Self { plugin_id })
    }
}

impl From<GetPluginHealthRequest> for proto::GetPluginHealthRequest {
    fn from(value: GetPluginHealthRequest) -> Self {
        Self {
            plugin_id: Some(value.plugin_id.into()),
        }
    }
}

impl ProtobufSerializable for GetPluginHealthRequest {
    type ProtobufMessage = proto::GetPluginHealthRequest;
}

//
// PluginHealthStatus
//

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PluginHealthStatus {
    NotDeployed,
    // These map to https://www.nomadproject.io/api-docs/jobs#status
    Pending,
    Running,
    Dead,
}

impl TryFrom<PluginHealthStatusProto> for PluginHealthStatus {
    type Error = SerDeError;

    fn try_from(value: PluginHealthStatusProto) -> Result<Self, Self::Error> {
        match value {
            PluginHealthStatusProto::Unspecified => {
                Err(SerDeError::UnknownVariant("PluginHealthStatus"))
            }
            PluginHealthStatusProto::NotDeployed => Ok(PluginHealthStatus::NotDeployed),
            PluginHealthStatusProto::Pending => Ok(PluginHealthStatus::Pending),
            PluginHealthStatusProto::Running => Ok(PluginHealthStatus::Running),
            PluginHealthStatusProto::Dead => Ok(PluginHealthStatus::Dead),
        }
    }
}

impl From<PluginHealthStatus> for PluginHealthStatusProto {
    fn from(value: PluginHealthStatus) -> Self {
        match value {
            PluginHealthStatus::NotDeployed => PluginHealthStatusProto::NotDeployed,
            PluginHealthStatus::Pending => PluginHealthStatusProto::Pending,
            PluginHealthStatus::Running => PluginHealthStatusProto::Running,
            PluginHealthStatus::Dead => PluginHealthStatusProto::Dead,
        }
    }
}

//
// GetPluginHealthResponse
//

#[derive(Debug, Clone, PartialEq)]
pub struct GetPluginHealthResponse {
    health_status: PluginHealthStatus,
}

impl GetPluginHealthResponse {
    pub fn new(health_status: PluginHealthStatus) -> Self {
        Self { health_status }
    }

    pub fn health_status(&self) -> PluginHealthStatus {
        self.health_status
    }
}

impl type_url::TypeUrl for GetPluginHealthResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.GetPluginHealthResponse";
}

impl TryFrom<proto::GetPluginHealthResponse> for GetPluginHealthResponse {
    type Error = SerDeError;

    fn try_from(value: proto::GetPluginHealthResponse) -> Result<Self, Self::Error> {
        // Note that the `.some_enum()` has parens after!
        let health_status = value.health_status().try_into()?;
        Ok(Self { health_status })
    }
}

impl From<GetPluginHealthResponse> for proto::GetPluginHealthResponse {
    fn from(value: GetPluginHealthResponse) -> Self {
        let health_status: PluginHealthStatusProto = value.health_status.into();
        Self {
            health_status: health_status as i32,
        }
    }
}

impl ProtobufSerializable for GetPluginHealthResponse {
    type ProtobufMessage = proto::GetPluginHealthResponse;
}
