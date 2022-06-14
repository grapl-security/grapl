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
        let description = value.description;
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
            .ok_or(SerDeError::MissingField("created_time"))?;
        let created_time: SystemTime = created_time.try_into()?;

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
