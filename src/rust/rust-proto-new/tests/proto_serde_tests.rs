use std::fmt::Debug;

mod test_utils;
use proptest::prelude::*;
use rust_proto_new::{
    graplinc::{
        common::v1beta1::{
            Duration,
            SystemTime,
        },
        grapl::{
            api::pipeline_ingress::v1beta1::{
                PublishRawLogRequest,
                PublishRawLogResponse,
            },
            pipeline::v1beta2::Envelope,
        },
    },
    SerDe,
};
use test_utils::{
    serde::check_encode_decode_invariant,
    strategies,
};

fn envelopes<T>(inner_strategy: impl Strategy<Value = T>) -> impl Strategy<Value = Envelope<T>>
where
    T: SerDe + Debug,
{
    (strategies::pipeline::metadatas(), inner_strategy).prop_map(
        |(metadata, inner_message)| -> Envelope<T> {
            Envelope {
                metadata,
                inner_message,
            }
        },
    )
}

//
// PublishRawLogRequest
//

prop_compose! {
    fn publish_raw_log_requests()(
        event_source_id in strategies::uuids(),
        tenant_id in strategies::uuids(),
        log_event in strategies::bytes(256),
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
    fn publish_raw_log_responses()(
        created_time in any::<SystemTime>(),
    ) -> PublishRawLogResponse {
        PublishRawLogResponse {
            created_time,
        }
    }
}

//
// ---------------- helpers ---------------------------------------------------
//

//
// ---------------- protobuf tests ---------------------------------------------
//
// These tests check the encode-decode invariant (and possibly other invariants)
// of the transport objects this crate provides. These tests should use the
// proptest generators and helper functions (defined above) to establish these
// invariants.

proptest! {
    //
    // common
    //

    #[test]
    fn test_duration_encode_decode(duration in any::<Duration>()) {
        check_encode_decode_invariant(duration)
    }

    #[test]
    fn test_system_time_encode_decode(system_time in any::<SystemTime>()) {
        check_encode_decode_invariant(system_time)
    }

    #[test]
    fn test_uuid_encode_decode(uuid in strategies::uuids()) {
        check_encode_decode_invariant(uuid)
    }

    //
    // pipeline
    //

    #[test]
    fn test_metadata_encode_decode(metadata in strategies::pipeline::metadatas()) {
        check_encode_decode_invariant(metadata)
    }

    #[test]
    fn test_raw_log_encode_decode(raw_log in strategies::pipeline::raw_logs()) {
        check_encode_decode_invariant(raw_log)
    }

    #[test]
    fn test_v1_envelope_encode_decode(envelope in strategies::pipeline::v1_envelopes()) {
        check_encode_decode_invariant(envelope)
    }

    #[test]
    fn test_uuid_envelope_encode_decode(envelope in envelopes(strategies::uuids())) {
        check_encode_decode_invariant(envelope)
    }

    #[test]
    fn test_timestamp_envelope_encode_decode(envelope in envelopes(any::<SystemTime>())) {
        check_encode_decode_invariant(envelope)
    }

    #[test]
    fn test_duration_envelope_encode_decode(envelope in envelopes(any::<Duration>())) {
        check_encode_decode_invariant(envelope)
    }

    #[test]
    fn test_raw_log_envelope_encode_decode(envelope in envelopes(strategies::pipeline::raw_logs())) {
        check_encode_decode_invariant(envelope)
    }

    //
    // api.pipeline_ingress
    //

    #[test]
    fn test_publish_raw_log_request_encode_decode(
        publish_raw_log_request in publish_raw_log_requests()
    ) {
        check_encode_decode_invariant(publish_raw_log_request)
    }

    #[test]
    fn test_publish_raw_log_response_encode_decode(
        publish_raw_log_response in publish_raw_log_responses()
    ) {
        check_encode_decode_invariant(publish_raw_log_response)
    }

    // TODO: add more here as they're implemented
}
