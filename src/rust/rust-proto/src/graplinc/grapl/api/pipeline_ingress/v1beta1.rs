use bytes::Bytes;

pub mod client;
pub mod server;

use crate::{
    graplinc::common::v1beta1::{
        SystemTime,
        Uuid,
    },
    protobufs::graplinc::grapl::api::pipeline_ingress::v1beta1::{
        PublishRawLogRequest as PublishRawLogRequestProto,
        PublishRawLogResponse as PublishRawLogResponseProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

//
// PublishRawLogRequest
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishRawLogRequest {
    event_source_id: Uuid,
    tenant_id: Uuid,
    log_event: Bytes,
}

impl PublishRawLogRequest {
    pub fn new(event_source_id: Uuid, tenant_id: Uuid, log_event: Bytes) -> Self {
        Self {
            event_source_id,
            tenant_id,
            log_event,
        }
    }

    pub fn event_source_id(&self) -> Uuid {
        self.event_source_id
    }

    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    pub fn log_event(self) -> Bytes {
        self.log_event
    }
}

impl TryFrom<PublishRawLogRequestProto> for PublishRawLogRequest {
    type Error = SerDeError;

    fn try_from(request_proto: PublishRawLogRequestProto) -> Result<Self, Self::Error> {
        let event_source_id = request_proto
            .event_source_id
            .ok_or(SerDeError::MissingField("event_source_id"))?;

        let tenant_id = request_proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?;

        Ok(PublishRawLogRequest {
            event_source_id: event_source_id.into(),
            tenant_id: tenant_id.into(),
            log_event: request_proto.log_event,
        })
    }
}

impl From<PublishRawLogRequest> for PublishRawLogRequestProto {
    fn from(request: PublishRawLogRequest) -> Self {
        PublishRawLogRequestProto {
            event_source_id: Some(request.event_source_id.into()),
            tenant_id: Some(request.tenant_id.into()),
            log_event: request.log_event,
        }
    }
}

impl type_url::TypeUrl for PublishRawLogRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.pipeline_ingress.v1beta1.PublishRawLogRequest";
}

impl serde_impl::ProtobufSerializable for PublishRawLogRequest {
    type ProtobufMessage = PublishRawLogRequestProto;
}

//
// PublishRawLogResponse
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishRawLogResponse {
    created_time: SystemTime,
}

impl PublishRawLogResponse {
    pub fn new(created_time: SystemTime) -> Self {
        Self { created_time }
    }

    /// build a response with created_time set to SystemTime::now()
    pub fn ok() -> Self {
        PublishRawLogResponse {
            created_time: SystemTime::now(),
        }
    }

    pub fn created_time(&self) -> SystemTime {
        self.created_time
    }
}

impl TryFrom<PublishRawLogResponseProto> for PublishRawLogResponse {
    type Error = SerDeError;

    fn try_from(response_proto: PublishRawLogResponseProto) -> Result<Self, Self::Error> {
        let created_time = response_proto
            .created_time
            .ok_or(SerDeError::MissingField("created_time"))?;

        Ok(PublishRawLogResponse {
            created_time: created_time.try_into()?,
        })
    }
}

impl TryFrom<PublishRawLogResponse> for PublishRawLogResponseProto {
    type Error = SerDeError;

    fn try_from(response: PublishRawLogResponse) -> Result<Self, Self::Error> {
        Ok(PublishRawLogResponseProto {
            created_time: Some(response.created_time.try_into()?),
        })
    }
}

impl type_url::TypeUrl for PublishRawLogResponse {
    const TYPE_URL: &'static str =
        "grapsecurity.com/graplinc.grapl.api.pipeline_ingress.v1beta1.PublishRawLogResponse";
}

impl serde_impl::ProtobufSerializable for PublishRawLogResponse {
    type ProtobufMessage = PublishRawLogResponseProto;
}
