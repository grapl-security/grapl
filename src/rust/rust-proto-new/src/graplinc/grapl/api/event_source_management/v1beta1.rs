use std::fmt::Debug;

use crate::{
    graplinc::common::v1beta1::{
        SystemTime,
        Uuid,
    },
    protobufs::graplinc::grapl::api::event_source_management::v1beta1 as proto,
    serde_impl::ProtobufSerializable,
    type_url,
    SerDeError,
};

//////////////////// CreateEventSourceRequest ////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct CreateEventSourceRequest {
    pub display_name: String,
    pub description: String,
    pub tenant_id: Uuid,
}

impl ProtobufSerializable for CreateEventSourceRequest {
    type ProtobufMessage = proto::CreateEventSourceRequest;
}

impl type_url::TypeUrl for CreateEventSourceRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.event_source_management.v1beta1.CreateEventSourceRequest";
}

impl TryFrom<proto::CreateEventSourceRequest> for CreateEventSourceRequest {
    type Error = SerDeError;

    fn try_from(value: proto::CreateEventSourceRequest) -> Result<Self, Self::Error> {
        let display_name = value.display_name;
        if display_name.is_empty() {
            return Err(SerDeError::MissingField("display_name"));
        }
        let description = value.description;
        if description.is_empty() {
            return Err(SerDeError::MissingField("description"));
        }
        let tenant_id = value
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?
            .into();

        Ok(Self {
            display_name,
            description,
            tenant_id,
        })
    }
}

impl From<CreateEventSourceRequest> for proto::CreateEventSourceRequest {
    fn from(value: CreateEventSourceRequest) -> Self {
        Self {
            display_name: value.display_name,
            description: value.description,
            tenant_id: Some(value.tenant_id.into()),
        }
    }
}

//////////////////// CreateEventSourceResponse ////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct CreateEventSourceResponse {
    pub event_source_id: Uuid,
    pub created_time: SystemTime,
}

impl ProtobufSerializable for CreateEventSourceResponse {
    type ProtobufMessage = proto::CreateEventSourceResponse;
}

impl type_url::TypeUrl for CreateEventSourceResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.event_source_management.v1beta1.CreateEventSourceResponse";
}

impl TryFrom<proto::CreateEventSourceResponse> for CreateEventSourceResponse {
    type Error = SerDeError;

    fn try_from(value: proto::CreateEventSourceResponse) -> Result<Self, Self::Error> {
        let event_source_id = value
            .event_source_id
            .ok_or(SerDeError::MissingField("event_source_id"))?
            .into();
        let created_time = value
            .created_time
            .ok_or(SerDeError::MissingField("created_time"))?
            .try_into()?;

        Ok(Self {
            event_source_id,
            created_time,
        })
    }
}

impl TryFrom<CreateEventSourceResponse> for proto::CreateEventSourceResponse {
    type Error = SerDeError;
    fn try_from(value: CreateEventSourceResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            event_source_id: Some(value.event_source_id.into()),
            created_time: Some(value.created_time.try_into()?),
        })
    }
}

//////////////////// UpdateEventSourceRequest ////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateEventSourceRequest {
    pub event_source_id: Uuid,
    pub display_name: String,
    pub description: String,
    pub active: bool,
}

impl ProtobufSerializable for UpdateEventSourceRequest {
    type ProtobufMessage = proto::UpdateEventSourceRequest;
}

impl type_url::TypeUrl for UpdateEventSourceRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.event_source_management.v1beta1.UpdateEventSourceRequest";
}

impl TryFrom<proto::UpdateEventSourceRequest> for UpdateEventSourceRequest {
    type Error = SerDeError;

    fn try_from(value: proto::UpdateEventSourceRequest) -> Result<Self, Self::Error> {
        let event_source_id = value
            .event_source_id
            .ok_or(SerDeError::MissingField("event_source_id"))?
            .into();
        let display_name = value.display_name;
        if display_name.is_empty() {
            return Err(SerDeError::MissingField("display_name"));
        }
        let description = value.description;
        if description.is_empty() {
            return Err(SerDeError::MissingField("description"));
        }
        let active = value.active;

        Ok(Self {
            event_source_id,
            display_name,
            description,
            active,
        })
    }
}

impl From<UpdateEventSourceRequest> for proto::UpdateEventSourceRequest {
    fn from(value: UpdateEventSourceRequest) -> Self {
        Self {
            event_source_id: Some(value.event_source_id.into()),
            display_name: value.display_name,
            description: value.description,
            active: value.active,
        }
    }
}

//////////////////// UpdateEventSourceResponse ////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateEventSourceResponse {
    pub event_source_id: Uuid,
    pub last_updated_time: SystemTime,
}

impl ProtobufSerializable for UpdateEventSourceResponse {
    type ProtobufMessage = proto::UpdateEventSourceResponse;
}

impl type_url::TypeUrl for UpdateEventSourceResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.event_source_management.v1beta1.UpdateEventSourceResponse";
}

impl TryFrom<proto::UpdateEventSourceResponse> for UpdateEventSourceResponse {
    type Error = SerDeError;

    fn try_from(value: proto::UpdateEventSourceResponse) -> Result<Self, Self::Error> {
        let event_source_id = value
            .event_source_id
            .ok_or(SerDeError::MissingField("event_source_id"))?
            .into();
        let last_updated_time = value
            .last_updated_time
            .ok_or(SerDeError::MissingField("last_updated_time"))?
            .try_into()?;

        Ok(Self {
            event_source_id,
            last_updated_time,
        })
    }
}

impl TryFrom<UpdateEventSourceResponse> for proto::UpdateEventSourceResponse {
    type Error = SerDeError;
    fn try_from(value: UpdateEventSourceResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            event_source_id: Some(value.event_source_id.into()),
            last_updated_time: Some(value.last_updated_time.try_into()?),
        })
    }
}

//////////////////// GetEventSourceRequest ////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct GetEventSourceRequest {
    pub event_source_id: Uuid,
}

impl ProtobufSerializable for GetEventSourceRequest {
    type ProtobufMessage = proto::GetEventSourceRequest;
}

impl type_url::TypeUrl for GetEventSourceRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.event_source_management.v1beta1.GetEventSourceRequest";
}

impl TryFrom<proto::GetEventSourceRequest> for GetEventSourceRequest {
    type Error = SerDeError;

    fn try_from(value: proto::GetEventSourceRequest) -> Result<Self, Self::Error> {
        let event_source_id = value
            .event_source_id
            .ok_or(SerDeError::MissingField("event_source_id"))?
            .into();

        Ok(Self { event_source_id })
    }
}

impl TryFrom<GetEventSourceRequest> for proto::GetEventSourceRequest {
    type Error = SerDeError;
    fn try_from(value: GetEventSourceRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            event_source_id: Some(value.event_source_id.into()),
        })
    }
}

//////////////////// EventSource ////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct EventSource {
    pub tenant_id: Uuid,
    pub event_source_id: Uuid,
    pub display_name: String,
    pub description: String,
    pub created_time: SystemTime,
    pub last_updated_time: SystemTime,
    pub active: bool,
}

impl ProtobufSerializable for EventSource {
    type ProtobufMessage = proto::EventSource;
}

impl type_url::TypeUrl for EventSource {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.event_source_management.v1beta1.EventSource";
}

impl TryFrom<proto::EventSource> for EventSource {
    type Error = SerDeError;

    fn try_from(value: proto::EventSource) -> Result<Self, Self::Error> {
        let event_source_id = value
            .event_source_id
            .ok_or(SerDeError::MissingField("event_source_id"))?
            .into();

        let tenant_id = value
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?
            .into();

        let display_name = value.display_name;
        if display_name.is_empty() {
            return Err(SerDeError::MissingField("display_name"));
        }

        let description = value.description;
        if description.is_empty() {
            return Err(SerDeError::MissingField("description"));
        }

        let created_time = value
            .created_time
            .ok_or(SerDeError::MissingField("created_time"))?
            .try_into()?;

        let last_updated_time = value
            .last_updated_time
            .ok_or(SerDeError::MissingField("last_updated_time"))?
            .try_into()?;

        let active = value.active;

        Ok(Self {
            event_source_id,
            tenant_id,
            display_name,
            description,
            created_time,
            last_updated_time,
            active,
        })
    }
}

impl TryFrom<EventSource> for proto::EventSource {
    type Error = SerDeError;
    fn try_from(value: EventSource) -> Result<Self, Self::Error> {
        Ok(Self {
            event_source_id: Some(value.event_source_id.into()),
            tenant_id: Some(value.tenant_id.into()),
            display_name: value.display_name,
            description: value.description,
            created_time: Some(value.created_time.try_into()?),
            last_updated_time: Some(value.last_updated_time.try_into()?),
            active: value.active,
        })
    }
}

//////////////////// GetEventSourceResponse ////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct GetEventSourceResponse {
    pub event_source: EventSource,
}

impl ProtobufSerializable for GetEventSourceResponse {
    type ProtobufMessage = proto::GetEventSourceResponse;
}

impl type_url::TypeUrl for GetEventSourceResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.event_source_management.v1beta1.GetEventSourceResponse";
}

impl TryFrom<proto::GetEventSourceResponse> for GetEventSourceResponse {
    type Error = SerDeError;

    fn try_from(value: proto::GetEventSourceResponse) -> Result<Self, Self::Error> {
        let event_source = value
            .event_source
            .ok_or(SerDeError::MissingField("event_source"))?
            .try_into()?;

        Ok(Self { event_source })
    }
}

impl TryFrom<GetEventSourceResponse> for proto::GetEventSourceResponse {
    type Error = SerDeError;
    fn try_from(value: GetEventSourceResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            event_source: Some(value.event_source.try_into()?),
        })
    }
}
