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

pub trait HasAssociatedProto<T> {
    type Proto: From<T> + Message + Default + TryInto<T, Error = SerDeError>;
}
impl<T> SerDe for T
where
    T: HasAssociatedProto<T>,
    T: type_url::TypeUrl + Clone + std::fmt::Debug,
{
    fn serialize(self: T) -> Result<Bytes, SerDeError> {
        let proto = T::Proto::from(self);
        let mut buf = BytesMut::with_capacity(proto.encoded_len());
        proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: bytes::Buf,
        Self: Sized,
    {
        let proto: T::Proto = Message::decode(buf)?;
        proto.try_into()
    }
}
