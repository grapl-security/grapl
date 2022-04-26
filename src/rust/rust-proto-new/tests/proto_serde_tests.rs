mod test_utils;
use proptest::prelude::*;
use rust_proto_new::graplinc::common::v1beta1::{
    Duration,
    SystemTime,
};
use test_utils::{
    serde::check_encode_decode_invariant,
    strategies,
};

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
    fn test_uuid_envelope_encode_decode(envelope in strategies::pipeline::envelopes(strategies::uuids())) {
        check_encode_decode_invariant(envelope)
    }

    #[test]
    fn test_timestamp_envelope_encode_decode(envelope in strategies::pipeline::envelopes(any::<SystemTime>())) {
        check_encode_decode_invariant(envelope)
    }

    #[test]
    fn test_duration_envelope_encode_decode(envelope in strategies::pipeline::envelopes(any::<Duration>())) {
        check_encode_decode_invariant(envelope)
    }

    #[test]
    fn test_raw_log_envelope_encode_decode(envelope in strategies::pipeline::envelopes(strategies::pipeline::raw_logs())) {
        check_encode_decode_invariant(envelope)
    }

    //
    // api.pipeline_ingress
    //

    #[test]
    fn test_publish_raw_log_request_encode_decode(
        publish_raw_log_request in strategies::pipeline_ingress::publish_raw_log_requests()
    ) {
        check_encode_decode_invariant(publish_raw_log_request)
    }

    #[test]
    fn test_publish_raw_log_response_encode_decode(
        publish_raw_log_response in strategies::pipeline_ingress::publish_raw_log_responses()
    ) {
        check_encode_decode_invariant(publish_raw_log_response)
    }

    // TODO: add more here as they're implemented
}
