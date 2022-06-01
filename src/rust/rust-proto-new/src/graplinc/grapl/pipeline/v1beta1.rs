use std::time::SystemTimeError;

use bytes::Bytes;

use crate::{
    graplinc::common::v1beta1::{
        SystemTime,
        Uuid,
    },
    protobufs::graplinc::grapl::pipeline::v1beta1::{
        Envelope as EnvelopeProto,
        Metadata as MetadataProto,
        RawLog as RawLogProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

//
// Metadata
//

#[derive(Debug, PartialEq, Clone)]
pub struct Metadata {
    pub tenant_id: Uuid,
    pub trace_id: Uuid,
    pub retry_count: u32,
    pub created_time: SystemTime,
    pub last_updated_time: SystemTime,
    pub event_source_id: Uuid,
}

impl Metadata {
    pub fn new(
        tenant_id: Uuid,
        trace_id: Uuid,
        retry_count: u32,
        created_time: SystemTime,
        last_updated_time: SystemTime,
        event_source_id: Uuid,
    ) -> Self {
        Metadata {
            tenant_id,
            trace_id,
            retry_count,
            created_time,
            last_updated_time,
            event_source_id,
        }
    }

    pub fn create_from(other: Metadata) -> Self {
        let now = SystemTime::now();
        Metadata::new(
            other.tenant_id,
            other.trace_id,
            other.retry_count,
            now,
            now,
            other.event_source_id,
        )
    }

    pub fn retry(other: Metadata) -> Self {
        Metadata::new(
            other.tenant_id,
            other.trace_id,
            other.retry_count + 1,
            other.created_time,
            SystemTime::now(),
            other.event_source_id,
        )
    }
}

impl TryFrom<MetadataProto> for Metadata {
    type Error = SerDeError;

    fn try_from(metadata_proto: MetadataProto) -> Result<Self, Self::Error> {
        let tenant_id = metadata_proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?;

        let trace_id = metadata_proto
            .trace_id
            .ok_or(SerDeError::MissingField("trace_id"))?;

        let created_time = metadata_proto
            .created_time
            .ok_or(SerDeError::MissingField("created_time"))?;

        let last_updated_time = metadata_proto
            .last_updated_time
            .ok_or(SerDeError::MissingField("last_updated_time"))?;

        let event_source_id = metadata_proto
            .event_source_id
            .ok_or(SerDeError::MissingField("event_source_id"))?;

        Ok(Metadata {
            tenant_id: tenant_id.into(),
            trace_id: trace_id.into(),
            retry_count: metadata_proto.retry_count,
            created_time: created_time.try_into()?,
            last_updated_time: last_updated_time.try_into()?,
            event_source_id: event_source_id.into(),
        })
    }
}

impl TryFrom<Metadata> for MetadataProto {
    type Error = SystemTimeError;

    fn try_from(metadata: Metadata) -> Result<Self, Self::Error> {
        Ok(MetadataProto {
            tenant_id: Some(metadata.tenant_id.into()),
            trace_id: Some(metadata.trace_id.into()),
            retry_count: metadata.retry_count,
            created_time: Some(metadata.created_time.try_into()?),
            last_updated_time: Some(metadata.last_updated_time.try_into()?),
            event_source_id: Some(metadata.event_source_id.into()),
        })
    }
}

impl type_url::TypeUrl for Metadata {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.pipeline.v1beta1.Metadata";
}

impl serde_impl::ProtobufSerializable for Metadata {
    type ProtobufMessage = MetadataProto;
}

//
// Envelope
//

#[derive(Debug, PartialEq, Clone)]
pub struct Envelope {
    pub metadata: Metadata,
    pub inner_type: String,
    pub inner_message: Bytes,
}

impl TryFrom<EnvelopeProto> for Envelope {
    type Error = SerDeError;

    fn try_from(envelope_proto: EnvelopeProto) -> Result<Self, Self::Error> {
        let metadata = envelope_proto
            .metadata
            .ok_or(SerDeError::MissingField("metadata"))?;

        Ok(Envelope {
            metadata: metadata.try_into()?,
            inner_type: envelope_proto.inner_type,
            inner_message: Bytes::from(envelope_proto.inner_message),
        })
    }
}

impl TryFrom<Envelope> for EnvelopeProto {
    type Error = SerDeError;

    fn try_from(envelope: Envelope) -> Result<Self, Self::Error> {
        Ok(EnvelopeProto {
            metadata: Some(envelope.metadata.try_into()?),
            inner_type: envelope.inner_type,
            inner_message: envelope.inner_message.to_vec(),
        })
    }
}

impl type_url::TypeUrl for Envelope {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.pipeline.v1beta1.Envelope";
}

impl serde_impl::ProtobufSerializable for Envelope {
    type ProtobufMessage = EnvelopeProto;
}

//
// RawLog
//

#[derive(Debug, PartialEq, Clone)]
pub struct RawLog {
    pub log_event: Bytes,
}

impl RawLog {
    pub fn new(log_event: Bytes) -> Self {
        RawLog { log_event }
    }
}

impl From<RawLogProto> for RawLog {
    fn from(raw_log_proto: RawLogProto) -> Self {
        RawLog {
            log_event: Bytes::from(raw_log_proto.log_event),
        }
    }
}

impl From<RawLog> for RawLogProto {
    fn from(raw_log: RawLog) -> Self {
        RawLogProto {
            log_event: raw_log.log_event.to_vec(),
        }
    }
}

impl type_url::TypeUrl for RawLog {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.pipeline.v1beta1.RawLog";
}

impl serde_impl::ProtobufSerializable for RawLog {
    type ProtobufMessage = RawLogProto;
}
