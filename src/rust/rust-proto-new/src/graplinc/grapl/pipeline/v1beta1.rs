use std::time::SystemTimeError;

use bytes::{
    Buf,
    BufMut,
    Bytes,
};
use prost::Message;

use crate::{
    graplinc::common::v1beta1::{
        SystemTime,
        Uuid,
    },
    protobufs::graplinc::grapl::pipeline::v1beta1::{
        Envelope as _Envelope,
        Metadata as _Metadata,
        RawLog as _RawLog,
    },
    type_url,
    SerDe,
    SerDeError,
};

//
// Metadata
//

#[derive(Debug)]
pub struct Metadata {
    tenant_id: Uuid,
    trace_id: Uuid,
    retry_count: u32,
    created_time: SystemTime,
    last_updated_time: SystemTime,
    event_source_id: Uuid,
}

impl TryFrom<_Metadata> for Metadata {
    type Error = SerDeError;

    fn try_from(metadata_proto: _Metadata) -> Result<Self, Self::Error> {
        let tenant_id = if let Some(tenant_id) = metadata_proto.tenant_id {
            tenant_id
        } else {
            return Err(SerDeError::MissingField("tenant_id absent".to_string()));
        };

        let trace_id = if let Some(trace_id) = metadata_proto.trace_id {
            trace_id
        } else {
            return Err(SerDeError::MissingField("trace_id absent".to_string()));
        };

        let created_time = if let Some(created_time) = metadata_proto.created_time {
            created_time
        } else {
            return Err(SerDeError::MissingField("created_time absent".to_string()));
        };

        let last_updated_time = if let Some(last_updated_time) = metadata_proto.last_updated_time {
            last_updated_time
        } else {
            return Err(SerDeError::MissingField(
                "last_updated_time absent".to_string(),
            ));
        };

        let event_source_id = if let Some(event_source_id) = metadata_proto.event_source_id {
            event_source_id
        } else {
            return Err(SerDeError::MissingField(
                "event_source_id absent".to_string(),
            ));
        };

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

impl TryFrom<Metadata> for _Metadata {
    type Error = SystemTimeError;

    fn try_from(metadata: Metadata) -> Result<Self, Self::Error> {
        Ok(_Metadata {
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

impl SerDe for Metadata {
    fn serialize<B>(self, buf: &mut B) -> Result<(), SerDeError>
    where
        B: BufMut,
    {
        _Metadata::try_from(self)?.encode(buf)?;
        Ok(())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let metadata_proto: _Metadata = Message::decode(buf)?;
        Ok(metadata_proto.try_into()?)
    }
}

//
// Envelope
//

#[derive(Debug)]
pub struct Envelope {
    metadata: Metadata,
    inner_type: String,
    inner_message: Bytes,
}

impl TryFrom<_Envelope> for Envelope {
    type Error = SerDeError;

    fn try_from(envelope_proto: _Envelope) -> Result<Self, Self::Error> {
        let metadata = if let Some(metadata) = envelope_proto.metadata {
            metadata
        } else {
            return Err(SerDeError::MissingField("metadata absent".to_string()));
        };

        Ok(Envelope {
            metadata: metadata.try_into()?,
            inner_type: envelope_proto.inner_type,
            inner_message: Bytes::from(envelope_proto.inner_message),
        })
    }
}

impl TryFrom<Envelope> for _Envelope {
    type Error = SerDeError;

    fn try_from(envelope: Envelope) -> Result<Self, Self::Error> {
        Ok(_Envelope {
            metadata: Some(envelope.metadata.try_into()?),
            inner_type: envelope.inner_type,
            inner_message: envelope.inner_message.to_vec(),
        })
    }
}

impl type_url::TypeUrl for Envelope {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.pipeline.v1beta1.Envelope";
}

impl SerDe for Envelope {
    fn serialize<B>(self, buf: &mut B) -> Result<(), SerDeError>
    where
        B: BufMut,
    {
        _Envelope::try_from(self)?.encode(buf)?;
        Ok(())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let envelope_proto: _Envelope = Message::decode(buf)?;
        Ok(envelope_proto.try_into()?)
    }
}

//
// RawLog
//

#[derive(Debug)]
pub struct RawLog {
    log_event: Bytes,
}

impl From<_RawLog> for RawLog {
    fn from(raw_log_proto: _RawLog) -> Self {
        RawLog {
            log_event: Bytes::from(raw_log_proto.log_event),
        }
    }
}

impl From<RawLog> for _RawLog {
    fn from(raw_log: RawLog) -> Self {
        _RawLog {
            log_event: raw_log.log_event.to_vec(),
        }
    }
}

impl type_url::TypeUrl for RawLog {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.pipeline.v1beta1.RawLog";
}

impl SerDe for RawLog {
    fn serialize<B>(self, buf: &mut B) -> Result<(), SerDeError>
    where
        B: BufMut,
    {
        _RawLog::from(self).encode(buf)?;
        Ok(())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let raw_log_proto: _RawLog = Message::decode(buf)?;
        Ok(raw_log_proto.into())
    }
}
