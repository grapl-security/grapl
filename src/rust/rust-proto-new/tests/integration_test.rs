use bytes::{
    Buf,
    BufMut,
    BytesMut,
};
use proptest::prelude::*;
use rust_proto_new::{
    graplinc::{
        common::v1beta1::{
            Duration,
            SystemTime,
            Uuid,
        },
        grapl::pipeline::{
            v1beta1::{
                Envelope,
                Metadata,
                RawLog,
            },
            v1beta2::Envelope,
        },
    },
    SerDe,
    SerDeError,
};

fn check_encode_decode_invariant<T>(serializable: T)
where
    T: SerDe + PartialEq,
{
    let mut buf = BytesMut::new();
    let serialized = serializable.serialize(&mut buf);
    let deserialized = T::deserialize(buf).expect("deserialization failed");
    assert!(serializable == deserialized);
}

proptest! {
    #[test]
    fn test_duration_encode_decode() {
        todo!()
    }

    #[test]
    fn test_system_time_encode_decode() {
        todo!()
    }

    #[test]
    fn test_uuid_encode_decode() {
        todo!()
    }

    #[test]
    fn test_metadata_encode_decode() {
        todo!()
    }

    #[test]
    fn test_raw_log_encode_decode() {
        todo!()
    }

    #[test]
    fn test_envelope_encode_decode() {
        todo!()
    }

    #[test]
    fn test_new_envelope_encode_decode() {
        todo!()
    }
}
