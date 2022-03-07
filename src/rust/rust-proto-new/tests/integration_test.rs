use std::fmt::Debug;

use bytes::{
    Bytes,
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
                Envelope as EnvelopeV1,
                Metadata,
                RawLog,
            },
            v1beta2::Envelope,
        },
    },
    SerDe,
};

//
// ---------------- strategies ------------------------------------------------
//

//
// Bytes
//

fn bytes(size: usize) -> impl Strategy<Value = Bytes> {
    proptest::collection::vec(any::<u8>(), size).prop_map(Bytes::from)
}

//
// Uuid
//

prop_compose! {
    fn uuids()(
        int in any::<u128>()
    ) -> Uuid {
        Uuid::from_u128_le(int)
    }
}

//
// RawLog
//

prop_compose! {
    fn raw_logs()(
        log_event in bytes(256)
    ) -> RawLog {
        RawLog {
            log_event
        }
    }
}

//
// Metadata
//

prop_compose! {
    fn metadatas()(
        tenant_id in uuids(),
        trace_id in uuids(),
        retry_count in any::<u32>(),
        created_time in any::<SystemTime>(),
        last_updated_time in any::<SystemTime>(),
        event_source_id in uuids()
    ) -> Metadata {
        Metadata {
            tenant_id,
            trace_id,
            retry_count,
            created_time,
            last_updated_time,
            event_source_id,
        }
    }
}

//
// Envelope
//

prop_compose! {
    fn v1_envelopes()(
        metadata in metadatas(),
        inner_type in any::<String>(),
        inner_message in bytes(256),
    ) -> EnvelopeV1 {
        EnvelopeV1 {
            metadata,
            inner_type,
            inner_message
        }
    }
}

prop_compose! {
    fn envelopes()(
        metadata in metadatas(),
        inner_message in raw_logs()
    ) -> Envelope<RawLog> {
        Envelope {
            metadata,
            inner_message
        }
    }
}

//
// ---------------- helpers ---------------------------------------------------
//

// helper function to define a simple encode-decode invariant
// see: https://hypothesis.works/articles/encode-decode-invariant/
fn check_encode_decode_invariant<T>(serializable: T)
where
    T: SerDe + PartialEq + Clone + Debug,
{
    let mut buf = BytesMut::new();
    let cloned = serializable.clone();
    serializable
        .serialize(&mut buf)
        .expect("serialization failed");
    let deserialized = T::deserialize(buf).expect("deserialization failed");
    assert!(cloned == deserialized);
}

//
// ---------------- tests -----------------------------------------------------
//

proptest! {
    #[test]
    fn test_duration_encode_decode(duration in any::<Duration>()) {
        check_encode_decode_invariant(duration)
    }

    #[test]
    fn test_system_time_encode_decode(system_time in any::<SystemTime>()) {
        check_encode_decode_invariant(system_time)
    }

    #[test]
    fn test_uuid_encode_decode(uuid in uuids()) {
        check_encode_decode_invariant(uuid)
    }

    #[test]
    fn test_metadata_encode_decode(metadata in metadatas()) {
        check_encode_decode_invariant(metadata)
    }

    #[test]
    fn test_raw_log_encode_decode(raw_log in raw_logs()) {
        check_encode_decode_invariant(raw_log)
    }

    #[test]
    fn test_v1_envelope_encode_decode(envelope in v1_envelopes()) {
        check_encode_decode_invariant(envelope)
    }

    #[test]
    fn test_envelope_encode_decode(envelope in envelopes()) {
        check_encode_decode_invariant(envelope)
    }
}
