use bytes::Bytes;
use prost::Message;

use crate::{
    graplinc::common::v1beta1::{
        SystemTime,
        Uuid,
    },
    protobufs::graplinc::grapl::api::pipeline_ingress::v1beta1::{
        PublishRawLogsRequest as PublishRawLogsRequestProto,
        PublishRawLogsResponse as PublishRawLogsResponseProto,
    },
    type_url,
    SerDe,
    SerDeError,
};

//
// PublishRawLogsRequest
//

#[derive(Debug, Clone, PartialEq)]
pub struct PublishRawLogsRequest {
    pub event_source_id: Uuid,
    pub tenant_id: Uuid,
    pub log_event: Bytes,
}

impl TryFrom<PublishRawLogsRequestProto> for PublishRawLogsRequest {
    type Error = SerDeError;

    fn try_from(
        publish_raw_logs_request_proto: PublishRawLogsRequestProto,
    ) -> Result<Self, Self::Error> {
        let event_source_id = publish_raw_logs_request_proto
            .event_source_id
            .ok_or(SerDeError::MissingField("event_source_id".to_string()));

        let tenant_id = publish_raw_logs_request_proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id".to_string()));

        Ok(PublishRawLogsRequest {
            event_source_id: event_source_id?.into(),
            tenant_id: tenant_id?.into(),
            log_event: Bytes::from(publish_raw_logs_request_proto.log_event),
        })
    }
}

impl From<PublishRawLogsRequest> for PublishRawLogsRequestProto {
    fn from(publish_raw_logs_request: PublishRawLogsRequest) -> Self {
        PublishRawLogsRequestProto {
            event_source_id: Some(publish_raw_logs_request.event_source_id.into()),
            tenant_id: Some(publish_raw_logs_request.tenant_id.into()),
            log_event: publish_raw_logs_request.log_event.to_vec(),
        }
    }
}

impl type_url::TypeUrl for PublishRawLogsRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.pipeline_ingress.v1beta1.PublishRawLogsRequest";
}

impl SerDe for PublishRawLogsRequest {
    fn serialize<B>(self, buf: &mut B) -> Result<(), SerDeError>
    where
        B: bytes::BufMut,
    {
        PublishRawLogsRequestProto::from(self).encode(buf)?;
        Ok(())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: bytes::Buf,
        Self: Sized,
    {
        let publish_raw_logs_request_proto: PublishRawLogsRequestProto = Message::decode(buf)?;
        publish_raw_logs_request_proto.try_into()
    }
}

//
// PublishRawLogsResponse
//

#[derive(Debug, Clone, PartialEq)]
pub struct PublishRawLogsResponse {
    pub created_time: SystemTime,
}

impl TryFrom<PublishRawLogsResponseProto> for PublishRawLogsResponse {
    type Error = SerDeError;

    fn try_from(
        publish_raw_logs_response_proto: PublishRawLogsResponseProto,
    ) -> Result<Self, Self::Error> {
        let created_time = publish_raw_logs_response_proto
            .created_time
            .ok_or(SerDeError::MissingField("created_time".to_string()));

        Ok(PublishRawLogsResponse {
            created_time: created_time?.try_into()?,
        })
    }
}

impl TryFrom<PublishRawLogsResponse> for PublishRawLogsResponseProto {
    type Error = SerDeError;

    fn try_from(publish_raw_logs_response: PublishRawLogsResponse) -> Result<Self, Self::Error> {
        Ok(PublishRawLogsResponseProto {
            created_time: Some(publish_raw_logs_response.created_time.try_into()?),
        })
    }
}

impl type_url::TypeUrl for PublishRawLogsResponse {
    const TYPE_URL: &'static str =
        "grapsecurity.com/graplinc.grapl.api.pipeline_ingress.v1beta1.PublishRawLogsResponse";
}

impl SerDe for PublishRawLogsResponse {
    fn serialize<B>(self, buf: &mut B) -> Result<(), SerDeError>
    where
        B: bytes::BufMut,
    {
        PublishRawLogsResponseProto::try_from(self)?.encode(buf)?;
        Ok(())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: bytes::Buf,
        Self: Sized,
    {
        let publish_raw_logs_response_proto: PublishRawLogsResponseProto = Message::decode(buf)?;
        publish_raw_logs_response_proto.try_into()
    }
}
