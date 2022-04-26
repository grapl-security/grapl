use bytes::Bytes;
use proptest::prelude::*;
use uuid::Uuid;

//
// Bytes
//

pub fn bytes(size: usize) -> impl Strategy<Value = Bytes> {
    proptest::collection::vec(any::<u8>(), size).prop_map(Bytes::from)
}

//
// Uuid
//

prop_compose! {
    pub fn uuids()(
        int in any::<u128>()
    ) -> Uuid {
        Uuid::from_u128_le(int)
    }
}

pub mod pipeline {
    use std::time::SystemTime;

    use rust_proto_new::graplinc::grapl::pipeline::v1beta1::{
        Envelope as EnvelopeV1,
        Metadata,
        RawLog,
    };

    use super::*;

    //
    // RawLog
    //

    prop_compose! {
        pub fn raw_logs()(
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
        pub fn metadatas()(
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
        pub fn v1_envelopes()(
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
}
