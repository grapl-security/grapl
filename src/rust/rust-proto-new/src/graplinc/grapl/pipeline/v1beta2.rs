use bytes::{
    Buf,
    Bytes,
    BytesMut,
};
use prost::Message;
use prost_types::Any as AnyProto;

use crate::{
    graplinc::grapl::pipeline::v1beta1::Metadata,
    protobufs::graplinc::grapl::pipeline::v1beta2::NewEnvelope as NewEnvelopeProto,
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

impl<T> Envelope<T>
where
    T: SerDe,
{
    pub fn new(metadata: Metadata, inner_message: T) -> Self {
        Envelope {
            metadata,
            inner_message,
        }
    }
}

impl<T> TryFrom<NewEnvelopeProto> for Envelope<T>
where
    T: SerDe,
{
    type Error = SerDeError;

    fn try_from(envelope_proto: NewEnvelopeProto) -> Result<Self, Self::Error> {
        let metadata = envelope_proto
            .metadata
            .ok_or(SerDeError::MissingField("metadata"))?;

        if let Some(any_proto) = envelope_proto.inner_message {
            Ok(Envelope {
                metadata: metadata.try_into()?,
                inner_message: SerDe::deserialize(Bytes::from(any_proto.value))?,
            })
        } else {
            Err(SerDeError::MissingField("inner_message"))
        }
    }
}

impl<T> TryFrom<Envelope<T>> for NewEnvelopeProto
where
    T: SerDe,
{
    type Error = SerDeError;

    fn try_from(envelope: Envelope<T>) -> Result<Self, Self::Error> {
        Ok(NewEnvelopeProto {
            metadata: Some(envelope.metadata.try_into()?),
            inner_message: Some(AnyProto {
                type_url: T::TYPE_URL.to_string(),
                value: envelope.inner_message.serialize()?.to_vec(),
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
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let envelope_proto = NewEnvelopeProto::try_from(self)?;
        let mut buf = BytesMut::with_capacity(envelope_proto.encoded_len());
        envelope_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let envelope_proto: NewEnvelopeProto = Message::decode(buf)?;
        envelope_proto.try_into()
    }
}
