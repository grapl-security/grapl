use std::time::SystemTime;

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
    use std::fmt::Debug;

    use rust_proto_new::{
        graplinc::grapl::pipeline::{
            v1beta1::{
                Envelope as EnvelopeV1,
                Metadata,
                RawLog,
            },
            v1beta2::Envelope,
        },
        SerDe,
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

    pub fn envelopes<T>(
        inner_strategy: impl Strategy<Value = T>,
    ) -> impl Strategy<Value = Envelope<T>>
    where
        T: SerDe + Debug,
    {
        (metadatas(), inner_strategy).prop_map(|(metadata, inner_message)| -> Envelope<T> {
            Envelope {
                metadata,
                inner_message,
            }
        })
    }
}

pub mod pipeline_ingress {
    use rust_proto_new::graplinc::grapl::api::pipeline_ingress::v1beta1::{
        PublishRawLogRequest,
        PublishRawLogResponse,
    };

    use super::*;

    //
    // PublishRawLogRequest
    //

    prop_compose! {
        pub fn publish_raw_log_requests()(
            event_source_id in uuids(),
            tenant_id in uuids(),
            log_event in bytes(256),
        ) -> PublishRawLogRequest {
            PublishRawLogRequest {
                event_source_id,
                tenant_id,
                log_event
            }
        }
    }

    //
    // PublishRawLogResponse
    //

    prop_compose! {
        pub fn publish_raw_log_responses()(
            created_time in any::<SystemTime>(),
        ) -> PublishRawLogResponse {
            PublishRawLogResponse {
                created_time,
            }
        }
    }
}
