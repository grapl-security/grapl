use std::{
    fmt::Debug,
    time::SystemTime,
};

use bytes::Bytes;
use proto::create_plugin_request;

pub use crate::graplinc::grapl::api::plugin_registry::{
    v1beta1_client::PluginRegistryClient,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
// PluginDeployment
//

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginDeploymentStatus {
    Unspecified,
    Success,
    Fail,
}

impl From<proto::PluginDeploymentStatus> for PluginDeploymentStatus {
    fn from(proto_plugin_deployment_status: proto::PluginDeploymentStatus) -> Self {
        match proto_plugin_deployment_status {
            proto::PluginDeploymentStatus::Unspecified => Self::Unspecified,
            proto::PluginDeploymentStatus::Success => Self::Success,
            proto::PluginDeploymentStatus::Fail => Self::Fail,
        }
    }
}

impl From<PluginDeploymentStatus> for proto::PluginDeploymentStatus {
    fn from(plugin_deployment_status: PluginDeploymentStatus) -> Self {
        match plugin_deployment_status {
            PluginDeploymentStatus::Unspecified => Self::Unspecified,
            PluginDeploymentStatus::Success => Self::Success,
            PluginDeploymentStatus::Fail => Self::Fail,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginDeployment {
    plugin_id: uuid::Uuid,
    timestamp: SystemTime,
    status: PluginDeploymentStatus,
    deployed: bool,
}

impl PluginDeployment {
    pub fn new(
        plugin_id: uuid::Uuid,
        timestamp: SystemTime,
        status: PluginDeploymentStatus,
        deployed: bool,
    ) -> Self {
        Self {
            plugin_id,
            timestamp,
            status,
            deployed,
        }
    }

    pub fn plugin_id(&self) -> uuid::Uuid {
        self.plugin_id
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    pub fn status(&self) -> PluginDeploymentStatus {
        self.status
    }

    pub fn deployed(&self) -> bool {
        self.deployed
    }
}

impl type_url::TypeUrl for PluginDeployment {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.PluginDeployment";
}

impl TryFrom<proto::PluginDeployment> for PluginDeployment {
    type Error = SerDeError;

    fn try_from(proto_plugin_deployment: proto::PluginDeployment) -> Result<Self, Self::Error> {
        let status = proto_plugin_deployment.status().into();

        let plugin_id = proto_plugin_deployment
            .plugin_id
            .ok_or(SerDeError::MissingField("plugin_id"))?
            .into();

        let timestamp = proto_plugin_deployment
            .timestamp
            .ok_or(SerDeError::MissingField("timestamp"))?
            .try_into()?;

        Ok(Self {
            plugin_id,
            timestamp,
            status,
            deployed: proto_plugin_deployment.deployed,
        })
    }
}

impl TryFrom<PluginDeployment> for proto::PluginDeployment {
    type Error = SerDeError;

    fn try_from(plugin_deployment: PluginDeployment) -> Result<Self, Self::Error> {
        let status: proto::PluginDeploymentStatus = plugin_deployment.status().into();

        Ok(Self {
            plugin_id: Some(plugin_deployment.plugin_id().into()),
            timestamp: Some(plugin_deployment.timestamp().try_into()?),
            status: status as i32,
            deployed: plugin_deployment.deployed(),
        })
    }
}

impl ProtobufSerializable for PluginDeployment {
    type ProtobufMessage = proto::PluginDeployment;
}

//
// GetPluginDeploymentRequest
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetPluginDeploymentRequest {
    plugin_id: uuid::Uuid,
}

impl GetPluginDeploymentRequest {
    pub fn new(plugin_id: uuid::Uuid) -> Self {
        Self { plugin_id }
    }

    pub fn plugin_id(&self) -> uuid::Uuid {
        self.plugin_id
    }
}

impl type_url::TypeUrl for GetPluginDeploymentRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.GetPluginDeploymentRequest";
}

impl TryFrom<proto::GetPluginDeploymentRequest> for GetPluginDeploymentRequest {
    type Error = SerDeError;

    fn try_from(
        proto_get_plugin_deployment_request: proto::GetPluginDeploymentRequest,
    ) -> Result<Self, Self::Error> {
        let plugin_id = proto_get_plugin_deployment_request
            .plugin_id
            .ok_or(SerDeError::MissingField("plugin_id"))?
            .into();

        Ok(Self { plugin_id })
    }
}

impl From<GetPluginDeploymentRequest> for proto::GetPluginDeploymentRequest {
    fn from(get_plugin_deployment_request: GetPluginDeploymentRequest) -> Self {
        Self {
            plugin_id: Some(get_plugin_deployment_request.plugin_id().into()),
        }
    }
}

impl ProtobufSerializable for GetPluginDeploymentRequest {
    type ProtobufMessage = proto::GetPluginDeploymentRequest;
}

//
// GetPluginDeploymentResponse
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetPluginDeploymentResponse {
    plugin_deployment: PluginDeployment,
}

impl GetPluginDeploymentResponse {
    pub fn new(plugin_deployment: PluginDeployment) -> Self {
        Self { plugin_deployment }
    }

    pub fn plugin_deployment(self) -> PluginDeployment {
        self.plugin_deployment
    }
}

impl TryFrom<proto::GetPluginDeploymentResponse> for GetPluginDeploymentResponse {
    type Error = SerDeError;

    fn try_from(
        proto_get_plugin_deployment_response: proto::GetPluginDeploymentResponse,
    ) -> Result<Self, Self::Error> {
        let plugin_deployment = proto_get_plugin_deployment_response
            .plugin_deployment
            .ok_or(SerDeError::MissingField("plugin_deployment"))?
            .try_into()?;

        Ok(Self { plugin_deployment })
    }
}

impl TryFrom<GetPluginDeploymentResponse> for proto::GetPluginDeploymentResponse {
    type Error = SerDeError;

    fn try_from(
        get_plugin_deployment_response: GetPluginDeploymentResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            plugin_deployment: Some(
                get_plugin_deployment_response
                    .plugin_deployment()
                    .try_into()?,
            ),
        })
    }
}

impl type_url::TypeUrl for GetPluginDeploymentResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.GetPluginDeploymentResponse";
}

impl ProtobufSerializable for GetPluginDeploymentResponse {
    type ProtobufMessage = proto::GetPluginDeploymentResponse;
}

//
// GetAnalyzersForTenantRequest
//

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
// ListPluginsRequest
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListPluginsRequest {
    tenant_id: uuid::Uuid,
    plugin_type: PluginType,
}

impl ListPluginsRequest {
    pub fn new(tenant_id: uuid::Uuid, plugin_type: PluginType) -> Self {
        Self {
            tenant_id,
            plugin_type,
        }
    }

    pub fn tenant_id(&self) -> uuid::Uuid {
        self.tenant_id
    }

    pub fn plugin_type(&self) -> PluginType {
        self.plugin_type
    }
}

impl type_url::TypeUrl for ListPluginsRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.ListPluginsRequest";
}

impl TryFrom<proto::ListPluginsRequest> for ListPluginsRequest {
    type Error = SerDeError;

    fn try_from(
        proto_list_plugins_request: proto::ListPluginsRequest,
    ) -> Result<Self, Self::Error> {
        let plugin_type = proto_list_plugins_request.plugin_type().try_into()?;

        let tenant_id = proto_list_plugins_request
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?
            .into();

        Ok(Self::new(tenant_id, plugin_type))
    }
}

impl From<ListPluginsRequest> for proto::ListPluginsRequest {
    fn from(list_plugins_request: ListPluginsRequest) -> Self {
        let plugin_type: proto::PluginType = list_plugins_request.plugin_type().into();
        Self {
            tenant_id: Some(list_plugins_request.tenant_id().into()),
            plugin_type: Some(plugin_type as i32),
        }
    }
}

impl ProtobufSerializable for ListPluginsRequest {
    type ProtobufMessage = proto::ListPluginsRequest;
}

//
// ListPluginResponse
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListPluginsResponse {
    plugins: Vec<GetPluginResponse>,
}

impl ListPluginsResponse {
    pub fn new(plugins: Vec<GetPluginResponse>) -> Self {
        Self { plugins }
    }

    pub fn plugins(self) -> Vec<GetPluginResponse> {
        self.plugins
    }
}

impl type_url::TypeUrl for ListPluginsResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_registry.v1beta1.ListPluginsResponse";
}

impl TryFrom<proto::ListPluginsResponse> for ListPluginsResponse {
    type Error = SerDeError;

    fn try_from(
        proto_list_plugins_response: proto::ListPluginsResponse,
    ) -> Result<Self, Self::Error> {
        let results: Result<Vec<GetPluginResponse>, SerDeError> = proto_list_plugins_response
            .plugins
            .iter()
            .map(|response| GetPluginResponse::try_from(response.clone()))
            .collect();

        match results {
            Ok(plugins) => Ok(Self { plugins }),
            Err(e) => Err(e),
        }
    }
}

impl From<ListPluginsResponse> for proto::ListPluginsResponse {
    fn from(list_plugins_response: ListPluginsResponse) -> Self {
        Self {
            plugins: list_plugins_response
                .plugins()
                .iter()
                .map(|response| proto::GetPluginResponse::from(response.clone()))
                .collect(),
        }
    }
}

impl ProtobufSerializable for ListPluginsResponse {
    type ProtobufMessage = proto::ListPluginsResponse;
}

//
// TearDownPluginRequest
//

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    // We may want a discrimination between "running-healthy" and "running-unhealthy"
}

impl TryFrom<proto::PluginHealthStatus> for PluginHealthStatus {
    type Error = SerDeError;

    fn try_from(value: proto::PluginHealthStatus) -> Result<Self, Self::Error> {
        match value {
            proto::PluginHealthStatus::Unspecified => {
                Err(SerDeError::UnknownVariant("PluginHealthStatus"))
            }
            proto::PluginHealthStatus::NotDeployed => Ok(PluginHealthStatus::NotDeployed),
            proto::PluginHealthStatus::Pending => Ok(PluginHealthStatus::Pending),
            proto::PluginHealthStatus::Running => Ok(PluginHealthStatus::Running),
            proto::PluginHealthStatus::Dead => Ok(PluginHealthStatus::Dead),
        }
    }
}

impl From<PluginHealthStatus> for proto::PluginHealthStatus {
    fn from(value: PluginHealthStatus) -> Self {
        match value {
            PluginHealthStatus::NotDeployed => proto::PluginHealthStatus::NotDeployed,
            PluginHealthStatus::Pending => proto::PluginHealthStatus::Pending,
            PluginHealthStatus::Running => proto::PluginHealthStatus::Running,
            PluginHealthStatus::Dead => proto::PluginHealthStatus::Dead,
        }
    }
}

//
// GetPluginHealthResponse
//

#[derive(Debug, Clone, PartialEq, Eq)]
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
        let health_status: proto::PluginHealthStatus = value.health_status.into();
        Self {
            health_status: health_status as i32,
        }
    }
}

impl ProtobufSerializable for GetPluginHealthResponse {
    type ProtobufMessage = proto::GetPluginHealthResponse;
}
