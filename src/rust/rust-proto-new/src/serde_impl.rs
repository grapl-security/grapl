use bytes::{
    Bytes,
    BytesMut,
};
use prost::Message;

use crate::{
    type_url,
    SerDe,
    SerDeError,
};

/// Example usage:
/// 
/// use crate::protobufs::graplinc::grapl::api::some_service::v1beta1 as proto,
/// pub struct SomeNativeType {...}
/// impl ProtobufSerializable<SomeNativeType> for SomeNativeType {
///    type Protobuf = proto::CorrespondingProtobufType;
/// }
pub(crate) trait ProtobufSerializable<T> {
    type ProtobufMessage: From<T> + TryInto<T, Error = SerDeError> + Message + Default;
}
impl<T> SerDe for T
where
    T: ProtobufSerializable<T>,
    T: type_url::TypeUrl + Clone + std::fmt::Debug,
{
    fn serialize(self: T) -> Result<Bytes, SerDeError> {
        let proto = T::ProtobufMessage::from(self);
        let mut buf = BytesMut::with_capacity(proto.encoded_len());
        proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: bytes::Buf,
        Self: Sized,
    {
        let proto: T::ProtobufMessage = Message::decode(buf)?;
        proto.try_into()
    }
}
