pub use crate::graplinc::grapl::api::plugin_registry::v1beta1::{
    plugin::PluginType as _PluginType,
    plugin_registry_service_client,
    plugin_registry_service_server,
    CreatePluginRequest as CreatePluginRequestProto,
    CreatePluginResponse as CreatePluginResponseProto,
    DeployPluginRequest as DeployPluginRequestProto,
    DeployPluginResponse as DeployPluginResponseProto,
    GetAnalyzersForTenantRequest as GetAnalyzersForTenantRequestProto,
    GetAnalyzersForTenantResponse as GetAnalyzersForTenantResponseProto,
    GetGeneratorsForEventSourceRequest as GetGeneratorsForEventSourceRequestProto,
    GetGeneratorsForEventSourceResponse as GetGeneratorsForEventSourceResponseProto,
    GetPluginRequest as GetPluginRequestProto,
    GetPluginResponse as GetPluginResponseProto,
    Plugin as _Plugin,
    TearDownPluginRequest as TearDownPluginRequestProto,
    TearDownPluginResponse as TearDownPluginResponseProto,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginRegistryDeserializationError {
    #[error("Missing a required field")]
    MissingRequiredField(&'static str),
    #[error("Empty field")]
    EmptyField(&'static str),
    #[error("Unknown variant")]
    UnknownVariant(&'static str),
}

pub enum PluginType {
    Generator,
    Analyzer,
}

impl TryFrom<_PluginType> for PluginType {
    type Error = PluginRegistryDeserializationError;

    fn try_from(value: _PluginType) -> Result<Self, Self::Error> {
        match value {
            _PluginType::Unspecified => Err(PluginRegistryDeserializationError::UnknownVariant(
                "PluginType",
            )),
            _PluginType::Generator => Ok(PluginType::Generator),
            _PluginType::Analyzer => Ok(PluginType::Analyzer),
        }
        // todo!()
    }
}

impl From<PluginType> for _PluginType {
    fn from(value: PluginType) -> Self {
        match value {
            PluginType::Generator => _PluginType::Generator,
            PluginType::Analyzer => _PluginType::Analyzer,
        }
    }
}

pub struct Plugin {
    /// unique identifier for this plugin
    pub plugin_id: uuid::Uuid,
    /// The type of the plugin
    pub plugin_type: PluginType,
}

impl TryFrom<_Plugin> for Plugin {
    type Error = PluginRegistryDeserializationError;

    fn try_from(value: _Plugin) -> Result<Self, Self::Error> {
        let plugin_type = value.plugin_type().try_into()?;
        let plugin_id = value
            .plugin_id
            .ok_or(PluginRegistryDeserializationError::MissingRequiredField(
                "Plugin.plugin_id",
            ))?
            .into();

        Ok(Self {
            plugin_id,
            plugin_type,
        })
    }
}

impl From<Plugin> for _Plugin {
    fn from(value: Plugin) -> Self {
        let plugin_type: _PluginType = value.plugin_type.into();
        Self {
            plugin_id: Some(value.plugin_id.into()),
            plugin_type: plugin_type as i32,
        }
    }
}

pub struct CreatePluginRequest {
    /// the actual plugin code
    pub plugin_artifact: Vec<u8>,
    /// Tenant that is deploying this plugin
    pub tenant_id: uuid::Uuid,
    /// The string value to display to a user, non-empty
    pub display_name: String,
}

impl TryFrom<CreatePluginRequestProto> for CreatePluginRequest {
    type Error = PluginRegistryDeserializationError;

    fn try_from(value: CreatePluginRequestProto) -> Result<Self, Self::Error> {
        let tenant_id = value
            .tenant_id
            .ok_or(PluginRegistryDeserializationError::MissingRequiredField(
                "CreatePluginRequest.tenant_id",
            ))?
            .into();
        let display_name = value.display_name;
        let plugin_artifact = value.plugin_artifact;

        if display_name.is_empty() {
            return Err(PluginRegistryDeserializationError::EmptyField(
                "CreatePluginRequest.display_name",
            ));
        }

        if plugin_artifact.is_empty() {
            return Err(PluginRegistryDeserializationError::EmptyField(
                "CreatePluginRequest.plugin_artifact",
            ));
        }

        Ok(Self {
            plugin_artifact,
            tenant_id,
            display_name,
        })
    }
}

impl From<CreatePluginRequest> for CreatePluginRequestProto {
    fn from(value: CreatePluginRequest) -> Self {
        Self {
            plugin_artifact: value.plugin_artifact,
            tenant_id: Some(value.tenant_id.into()),
            display_name: value.display_name,
        }
    }
}

pub struct CreatePluginResponse {
    /// The identity of the plugin that was created
    pub plugin_id: uuid::Uuid,
}

impl TryFrom<CreatePluginResponseProto> for CreatePluginResponse {
    type Error = PluginRegistryDeserializationError;

    fn try_from(value: CreatePluginResponseProto) -> Result<Self, Self::Error> {
        let plugin_id = value
            .plugin_id
            .ok_or(PluginRegistryDeserializationError::MissingRequiredField(
                "CreatePluginResponse.plugin_id",
            ))?
            .into();

        Ok(Self { plugin_id })
    }
}

impl From<CreatePluginResponse> for CreatePluginResponseProto {
    fn from(value: CreatePluginResponse) -> Self {
        Self {
            plugin_id: Some(value.plugin_id.into()),
        }
    }
}

pub struct DeployPluginRequest {
    pub plugin_id: uuid::Uuid,
}

impl TryFrom<DeployPluginRequestProto> for DeployPluginRequest {
    type Error = PluginRegistryDeserializationError;

    fn try_from(value: DeployPluginRequestProto) -> Result<Self, Self::Error> {
        let plugin_id = value
            .plugin_id
            .ok_or(PluginRegistryDeserializationError::MissingRequiredField(
                "DeployPluginRequest.plugin_id",
            ))?
            .into();

        Ok(Self { plugin_id })
    }
}

impl From<DeployPluginRequest> for DeployPluginRequestProto {
    fn from(value: DeployPluginRequest) -> Self {
        Self {
            plugin_id: Some(value.plugin_id.into()),
        }
    }
}

pub struct DeployPluginResponse {}

impl TryFrom<DeployPluginResponseProto> for DeployPluginResponse {
    type Error = PluginRegistryDeserializationError;

    fn try_from(_value: DeployPluginResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<DeployPluginResponse> for DeployPluginResponseProto {
    fn from(_value: DeployPluginResponse) -> Self {
        Self {}
    }
}

pub struct GetAnalyzersForTenantRequest {
    /// The tenant id for the tenant whose analyzers we wish to fetch
    pub tenant_id: uuid::Uuid,
}

impl TryFrom<GetAnalyzersForTenantRequestProto> for GetAnalyzersForTenantRequest {
    type Error = PluginRegistryDeserializationError;

    fn try_from(value: GetAnalyzersForTenantRequestProto) -> Result<Self, Self::Error> {
        let tenant_id = value
            .tenant_id
            .ok_or(PluginRegistryDeserializationError::MissingRequiredField(
                "GetAnalyzersForTenantRequest.tenant_id",
            ))?
            .into();

        Ok(Self { tenant_id })
    }
}

impl From<GetAnalyzersForTenantRequest> for GetAnalyzersForTenantRequestProto {
    fn from(value: GetAnalyzersForTenantRequest) -> Self {
        Self {
            tenant_id: Some(value.tenant_id.into()),
        }
    }
}

pub struct GetAnalyzersForTenantResponse {
    /// The plugin ids for the analyzers belonging to a tenant
    pub plugin_ids: Vec<uuid::Uuid>,
}

impl TryFrom<GetAnalyzersForTenantResponseProto> for GetAnalyzersForTenantResponse {
    type Error = PluginRegistryDeserializationError;

    fn try_from(value: GetAnalyzersForTenantResponseProto) -> Result<Self, Self::Error> {
        if value.plugin_ids.is_empty() {
            return Err(PluginRegistryDeserializationError::EmptyField(
                "GetAnalyzersForTenantResponse.plugin_ids",
            ));
        }
        let plugin_ids = value.plugin_ids.into_iter().map(uuid::Uuid::from).collect();

        Ok(Self { plugin_ids })
    }
}

impl From<GetAnalyzersForTenantResponse> for GetAnalyzersForTenantResponseProto {
    fn from(value: GetAnalyzersForTenantResponse) -> Self {
        Self {
            plugin_ids: value.plugin_ids.into_iter().map(uuid::Uuid::into).collect(),
        }
    }
}

pub struct GetGeneratorsForEventSourceRequest {
    /// The event source id
    pub event_source_id: uuid::Uuid,
}

impl TryFrom<GetGeneratorsForEventSourceRequestProto> for GetGeneratorsForEventSourceRequest {
    type Error = PluginRegistryDeserializationError;

    fn try_from(value: GetGeneratorsForEventSourceRequestProto) -> Result<Self, Self::Error> {
        let event_source_id = value
            .event_source_id
            .ok_or(PluginRegistryDeserializationError::MissingRequiredField(
                "GetGeneratorsForEventSourceRequest.event_source_id",
            ))?
            .into();

        Ok(Self { event_source_id })
    }
}

impl From<GetGeneratorsForEventSourceRequest> for GetGeneratorsForEventSourceRequestProto {
    fn from(value: GetGeneratorsForEventSourceRequest) -> Self {
        Self {
            event_source_id: Some(value.event_source_id.into()),
        }
    }
}

pub struct GetGeneratorsForEventSourceResponse {
    pub plugin_ids: Vec<uuid::Uuid>,
}

impl TryFrom<GetGeneratorsForEventSourceResponseProto> for GetGeneratorsForEventSourceResponse {
    type Error = PluginRegistryDeserializationError;

    fn try_from(value: GetGeneratorsForEventSourceResponseProto) -> Result<Self, Self::Error> {
        if value.plugin_ids.is_empty() {
            return Err(PluginRegistryDeserializationError::EmptyField(
                "GetGeneratorsForEventSourceResponse.plugin_ids",
            ));
        }
        let plugin_ids = value.plugin_ids.into_iter().map(uuid::Uuid::from).collect();

        Ok(Self { plugin_ids })
    }
}

impl From<GetGeneratorsForEventSourceResponse> for GetGeneratorsForEventSourceResponseProto {
    fn from(value: GetGeneratorsForEventSourceResponse) -> Self {
        Self {
            plugin_ids: value.plugin_ids.into_iter().map(uuid::Uuid::into).collect(),
        }
    }
}

pub struct GetPluginRequest {
    /// The identity of the plugin
    pub plugin_id: uuid::Uuid,
}

impl TryFrom<GetPluginRequestProto> for GetPluginRequest {
    type Error = PluginRegistryDeserializationError;

    fn try_from(value: GetPluginRequestProto) -> Result<Self, Self::Error> {
        let plugin_id = value
            .plugin_id
            .ok_or(PluginRegistryDeserializationError::MissingRequiredField(
                "GetPluginRequest.plugin_id",
            ))?
            .into();

        Ok(Self { plugin_id })
    }
}

impl From<GetPluginRequest> for GetPluginRequestProto {
    fn from(value: GetPluginRequest) -> Self {
        Self {
            plugin_id: Some(value.plugin_id.into()),
        }
    }
}

pub struct GetPluginResponse {
    pub plugin: Plugin,
}

impl TryFrom<GetPluginResponseProto> for GetPluginResponse {
    type Error = PluginRegistryDeserializationError;

    fn try_from(value: GetPluginResponseProto) -> Result<Self, Self::Error> {
        let plugin = value
            .plugin
            .ok_or(PluginRegistryDeserializationError::MissingRequiredField(
                "GetPluginResponse.plugin",
            ))?
            .try_into()?;

        Ok(Self { plugin })
    }
}

impl From<GetPluginResponse> for GetPluginResponseProto {
    fn from(value: GetPluginResponse) -> Self {
        Self {
            plugin: Some(value.plugin.into()),
        }
    }
}

pub struct TearDownPluginRequest {
    pub plugin_id: uuid::Uuid,
}

impl TryFrom<TearDownPluginRequestProto> for TearDownPluginRequest {
    type Error = PluginRegistryDeserializationError;

    fn try_from(value: TearDownPluginRequestProto) -> Result<Self, Self::Error> {
        let plugin_id = value
            .plugin_id
            .ok_or(PluginRegistryDeserializationError::MissingRequiredField(
                "TearDownPluginRequest.plugin_id",
            ))?
            .into();

        Ok(Self { plugin_id })
    }
}

impl From<TearDownPluginRequest> for TearDownPluginRequestProto {
    fn from(value: TearDownPluginRequest) -> Self {
        Self {
            plugin_id: Some(value.plugin_id.into()),
        }
    }
}

pub struct TearDownPluginResponse {}

impl TryFrom<TearDownPluginResponseProto> for TearDownPluginResponse {
    type Error = PluginRegistryDeserializationError;

    fn try_from(_value: TearDownPluginResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<TearDownPluginResponse> for TearDownPluginResponseProto {
    fn from(_: TearDownPluginResponse) -> Self {
        Self {}
    }
}
