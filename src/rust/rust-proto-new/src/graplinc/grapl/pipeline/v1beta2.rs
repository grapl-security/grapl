use bytes::{
    Buf,
    BufMut,
    Bytes,
    BytesMut,
};
use prost::Message;
use prost_types::Any as _Any;

use crate::{
    graplinc::grapl::pipeline::v1beta1::Metadata,
    protobufs::graplinc::grapl::pipeline::v1beta2::NewEnvelope as _NewEnvelope,
    type_url,
    SerDe,
    SerDeError,
};

#[derive(Debug, PartialEq, Clone)]
pub struct Envelope<T>
where
    T: SerDe,
{
    pub metadata: Metadata,
    pub inner_message: T,
}

impl<T> TryFrom<_NewEnvelope> for Envelope<T>
where
    T: SerDe,
{
    type Error = SerDeError;

    fn try_from(envelope_proto: _NewEnvelope) -> Result<Self, Self::Error> {
        let metadata = envelope_proto
            .metadata
            .ok_or(SerDeError::MissingField("metadata".to_string()));

        if let Some(any_proto) = envelope_proto.inner_message {
            Ok(Envelope {
                metadata: metadata?.try_into()?,
                inner_message: SerDe::deserialize(Bytes::from(any_proto.value))?,
            })
        } else {
            return Err(SerDeError::MissingField("inner_message".to_string()));
        }
    }
}

impl<T> TryFrom<Envelope<T>> for _NewEnvelope
where
    T: SerDe,
{
    type Error = SerDeError;

    fn try_from(envelope: Envelope<T>) -> Result<Self, Self::Error> {
        let mut buf = BytesMut::new();
        envelope.inner_message.serialize(&mut buf)?;

        Ok(_NewEnvelope {
            metadata: Some(envelope.metadata.try_into()?),
            inner_message: Some(_Any {
                type_url: T::TYPE_URL.to_string(),
                value: buf.to_vec(),
            }),
        })
    }
}

impl<T> type_url::TypeUrl for Envelope<T>
where
    T: SerDe,
{
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.pipeline.v1beta2.NewEnvelope";
}

impl<T> SerDe for Envelope<T>
where
    T: SerDe,
{
    fn serialize<B>(self, buf: &mut B) -> Result<(), SerDeError>
    where
        B: BufMut,
    {
        _NewEnvelope::try_from(self)?.encode(buf)?;
        Ok(())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let envelope_proto: _NewEnvelope = Message::decode(buf)?;
        Ok(envelope_proto.try_into()?)
    }
}
