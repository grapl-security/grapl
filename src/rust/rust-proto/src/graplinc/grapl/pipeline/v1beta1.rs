use bytes::Bytes;

use crate::{
    graplinc::common::v1beta1::{
        SystemTime,
        Uuid,
    },
    protobufs::{
        google::protobuf::Any as AnyProto,
        graplinc::grapl::pipeline::v1beta1::{
            Envelope as EnvelopeProto,
            RawLog as RawLogProto,
        },
    },
    serde_impl,
    type_url,
    SerDe,
    SerDeError,
};

//
// Envelope
//

#[derive(Debug, PartialEq, Clone)]
pub struct Envelope<T>
where
    T: SerDe,
{
    tenant_id: Uuid,
    trace_id: Uuid,
    retry_count: u32,
    created_time: SystemTime,
    last_updated_time: SystemTime,
    event_source_id: Uuid,
    inner_message: T,
}

impl<T> Envelope<T>
where
    T: SerDe,
{
    pub fn new(tenant_id: Uuid, trace_id: Uuid, event_source_id: Uuid, inner_message: T) -> Self {
        let now = SystemTime::now();
        Envelope {
            tenant_id,
            trace_id,
            retry_count: 0,
            created_time: now,
            last_updated_time: now,
            event_source_id,
            inner_message,
        }
    }

    pub fn increment_retry_count(&mut self) {
        self.retry_count += 1;
        self.last_updated_time = SystemTime::now();
    }

    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    pub fn trace_id(&self) -> Uuid {
        self.trace_id
    }

    pub fn retry_count(&self) -> u32 {
        self.retry_count
    }

    pub fn created_time(&self) -> SystemTime {
        self.created_time
    }

    pub fn last_updated_time(&self) -> SystemTime {
        self.last_updated_time
    }

    pub fn event_source_id(&self) -> Uuid {
        self.event_source_id
    }

    pub fn inner_message(self) -> T {
        self.inner_message
    }
}

impl<T> TryFrom<EnvelopeProto> for Envelope<T>
where
    T: SerDe,
{
    type Error = SerDeError;

    fn try_from(envelope_proto: EnvelopeProto) -> Result<Self, Self::Error> {
        let tenant_id = envelope_proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?;

        let trace_id = envelope_proto
            .trace_id
            .ok_or(SerDeError::MissingField("trace_id"))?;

        let created_time = envelope_proto
            .created_time
            .ok_or(SerDeError::MissingField("created_time"))?;

        let last_updated_time = envelope_proto
            .last_updated_time
            .ok_or(SerDeError::MissingField("last_updated_time"))?;

        let event_source_id = envelope_proto
            .event_source_id
            .ok_or(SerDeError::MissingField("event_source_id"))?;

        if let Some(any_proto) = envelope_proto.inner_message {
            Ok(Envelope {
                tenant_id: tenant_id.into(),
                trace_id: trace_id.into(),
                retry_count: envelope_proto.retry_count,
                created_time: created_time.try_into()?,
                last_updated_time: last_updated_time.try_into()?,
                event_source_id: event_source_id.into(),
                inner_message: SerDe::deserialize(any_proto.value)?,
            })
        } else {
            Err(SerDeError::MissingField("inner_message"))
        }
    }
}

impl<T> TryFrom<Envelope<T>> for EnvelopeProto
where
    T: SerDe,
{
    type Error = SerDeError;

    fn try_from(envelope: Envelope<T>) -> Result<Self, Self::Error> {
        Ok(EnvelopeProto {
            tenant_id: Some(envelope.tenant_id.into()),
            trace_id: Some(envelope.trace_id.into()),
            retry_count: envelope.retry_count,
            created_time: Some(envelope.created_time.try_into()?),
            last_updated_time: Some(envelope.last_updated_time.try_into()?),
            event_source_id: Some(envelope.event_source_id.into()),
            inner_message: Some(AnyProto {
                type_url: T::TYPE_URL.to_string(),
                value: envelope.inner_message.serialize()?,
            }),
        })
    }
}

impl<T> type_url::TypeUrl for Envelope<T>
where
    T: SerDe,
{
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.pipeline.v1beta1.Envelope";
}

impl<T> serde_impl::ProtobufSerializable for Envelope<T>
where
    T: SerDe,
{
    type ProtobufMessage = EnvelopeProto;
}

//
// RawLog
//

#[derive(Debug, PartialEq, Clone)]
pub struct RawLog {
    log_event: Bytes,
}

impl RawLog {
    pub fn new(log_event: Bytes) -> Self {
        RawLog { log_event }
    }

    pub fn log_event(self) -> Bytes {
        self.log_event
    }
}

impl From<RawLogProto> for RawLog {
    fn from(raw_log_proto: RawLogProto) -> Self {
        RawLog {
            log_event: raw_log_proto.log_event,
        }
    }
}

impl From<RawLog> for RawLogProto {
    fn from(raw_log: RawLog) -> Self {
        RawLogProto {
            log_event: raw_log.log_event,
        }
    }
}

impl type_url::TypeUrl for RawLog {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.pipeline.v1beta1.RawLog";
}

impl serde_impl::ProtobufSerializable for RawLog {
    type ProtobufMessage = RawLogProto;
}
