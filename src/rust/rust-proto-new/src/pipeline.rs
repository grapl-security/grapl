use bytes::{Buf, BufMut, Bytes};
use uuid::Uuid;
use crate::{SerDeError, SerDe};
use crate::graplinc::common::v1beta1::Uuid as _Uuid;
use crate::graplinc::grapl::pipeline::v1beta1::{
    Metadata as _Metadata,
    Envelope as _Envelope,
    //RawLog as _RawLog,
};

pub struct Metadata {
    tenant_id: Uuid,
    trace_id: Uuid,
    retry_count: u32
}

impl SerDe for Metadata {
    fn serialize(&self, buf: &mut dyn BufMut) -> Result<(), SerDeError> {
        todo!()
    }

    fn deserialize(buf: &dyn Buf) -> Result<Self, SerDeError> {
        todo!()
    }
}

pub struct Envelope<T> where T: SerDe {
    metadata: Metadata,
    inner_message: T
}

pub struct RawLog {
    fixme: Bytes
}
