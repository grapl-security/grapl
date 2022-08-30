mod test_utils;
use proptest::prelude::*;
use rust_proto::graplinc::common::v1beta1::{
    Duration,
    SystemTime,
};
use test_utils::{
    serde::{
        check_encode_decode_invariant,
        expect_serde_error,
    },
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
}

mod event_source {
    use strategies::event_source as st;

    use super::*;

    proptest! {
        #[test]
        fn test_encode_decode_create_event_source_request(
            value in st::create_event_source_requests()
        ) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_encode_decode_create_event_source_response(
            value in st::create_event_source_responses()
        ) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_encode_decode_update_event_source_request(
            value in st::update_event_source_requests()
        ) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_encode_decode_update_event_source_response(
            value in st::update_event_source_responses()
        ) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_encode_decode_get_event_source_request(
            value in st::get_event_source_requests()
        ) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_encode_decode_get_event_source_response(
            value in st::get_event_source_responses()
        ) {
            check_encode_decode_invariant(value)
        }
    }
}

mod graph {
    use strategies::graph as st;

    use super::*;

    proptest! {
        #[test]
        fn test_decrement_only_int_prop_encode_decode(
            decrement_only_int_prop in st::decrement_only_int_props()
        ) {
            check_encode_decode_invariant(decrement_only_int_prop)
        }

        #[test]
        fn test_decrement_only_uint_prop_encode_decode(
            decrement_only_uint_prop in st::decrement_only_uint_props()
        ) {
            check_encode_decode_invariant(decrement_only_uint_prop)
        }

        #[test]
        fn test_immutable_int_prop_encode_decode(
            immutable_int_prop in st::immutable_int_props()
        ) {
            check_encode_decode_invariant(immutable_int_prop)
        }

        #[test]
        fn test_immutable_str_prop_encode_decode(
            immutable_str_prop in st::immutable_str_props()
        ) {
            check_encode_decode_invariant(immutable_str_prop)
        }

        #[test]
        fn test_immutable_uint_prop_encode_decode(
            immutable_uint_prop in st::immutable_uint_props()
        ) {
            check_encode_decode_invariant(immutable_uint_prop)
        }

        #[test]
        fn test_increment_only_int_prop_encode_decode(
            increment_only_int_prop in st::increment_only_int_props()
        ) {
            check_encode_decode_invariant(increment_only_int_prop)
        }

        #[test]
        fn test_increment_only_uint_prop_encode_decode(
            increment_only_uint_prop in st::increment_only_uint_props()
        ) {
            check_encode_decode_invariant(increment_only_uint_prop)
        }

        #[test]
        fn test_edge_encode_decode(edge in st::edges()) {
            check_encode_decode_invariant(edge)
        }

        #[test]
        fn test_edge_list_encode_decode(edge_list in st::edge_lists()) {
            check_encode_decode_invariant(edge_list)
        }

        #[test]
        fn test_session_encode_decode(session in st::sessions()) {
            check_encode_decode_invariant(session)
        }

        #[test]
        fn test_static_encode_decode(static_ in st::statics()) {
            check_encode_decode_invariant(static_)
        }

        #[test]
        fn test_id_strategy_encode_decode(id_strategy in st::id_strategies()) {
            check_encode_decode_invariant(id_strategy)
        }

        #[test]
        fn test_node_property_encode_decode(
            node_property in st::node_properties()
        ) {
            check_encode_decode_invariant(node_property)
        }

        #[test]
        fn test_node_description_encode_decode(
            node_description in st::node_descriptions()
        ) {
            check_encode_decode_invariant(node_description)
        }

        #[test]
        fn test_graph_description_encode_decode(
            graph_description in st::graph_descriptions()
        ) {
            check_encode_decode_invariant(graph_description)
        }

        #[test]
        fn test_identified_node_encode_decode(
            identified_node in st::identified_nodes()
        ) {
            check_encode_decode_invariant(identified_node)
        }

        #[test]
        fn test_identified_graph_encode_decode(
            identified_graph in st::identified_graphs()
        ) {
            check_encode_decode_invariant(identified_graph)
        }

        #[test]
        fn test_identified_edge_encode_decode(identified_edge in st::identified_edges()) {
            check_encode_decode_invariant(identified_edge)
        }

        #[test]
        fn test_identified_edge_list_encode_decode(
            identified_edge_list in st::identified_edge_lists()
        ) {
            check_encode_decode_invariant(identified_edge_list)
        }

        #[test]
        fn test_lens_encode_decode(
            lens in st::lenses()
        ) {
            check_encode_decode_invariant(lens)
        }

        #[test]
        fn test_execution_hit_encode_decode(
            execution_hit in st::execution_hits()
        ) {
            check_encode_decode_invariant(execution_hit)
        }
    }
}

mod pipeline {
    use strategies::pipeline as st;

    use super::*;

    proptest! {
        #[test]
        fn test_raw_log_encode_decode(raw_log in st::raw_logs()) {
            check_encode_decode_invariant(raw_log)
        }

        #[test]
        fn test_uuid_envelope_encode_decode(
            envelope in st::envelopes(strategies::uuids())
        ) {
            check_encode_decode_invariant(envelope)
        }

        #[test]
        fn test_timestamp_envelope_encode_decode(
            envelope in st::envelopes(any::<SystemTime>())
        ) {
            check_encode_decode_invariant(envelope)
        }

        #[test]
        fn test_duration_envelope_encode_decode(
            envelope in st::envelopes(any::<Duration>())
        ) {
            check_encode_decode_invariant(envelope)
        }

        #[test]
        fn test_raw_log_envelope_encode_decode(
            envelope in st::envelopes(st::raw_logs())
        ){
            check_encode_decode_invariant(envelope)
        }
    }
}

mod pipeline_ingress {
    use strategies::pipeline_ingress as st;

    use super::*;

    proptest! {
        #[test]
        fn test_publish_raw_log_request_encode_decode(
            publish_raw_log_request in st::publish_raw_log_requests()
        ) {
            check_encode_decode_invariant(publish_raw_log_request)
        }

        #[test]
        fn test_publish_raw_log_response_encode_decode(
            publish_raw_log_response in st::publish_raw_log_responses()
        ) {
            check_encode_decode_invariant(publish_raw_log_response)
        }
    }
}

mod plugin_registry {
    use strategies::plugin_registry as pr_strats;

    use super::*;

    proptest! {
        #[test]
        fn test_serde_plugin_metadatas(value in pr_strats::plugin_metadatas()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_serde_create_plugin_requests(value in pr_strats::create_plugin_requests()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_serde_create_plugin_responses(value in pr_strats::create_plugin_responses()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_serde_get_analyzers_for_tenant_requests(value in pr_strats::get_analyzers_for_tenant_requests()) {
            check_encode_decode_invariant(value)
        }
        #[test]
        fn test_serde_get_analyzers_for_tenant_responses(value in pr_strats::get_analyzers_for_tenant_responses()) {
            check_encode_decode_invariant(value)
        }
        #[test]
        fn test_serde_deploy_plugin_requests(value in pr_strats::deploy_plugin_requests()) {
            check_encode_decode_invariant(value)
        }
        #[test]
        fn test_serde_deploy_plugin_responses(value in pr_strats::deploy_plugin_responses()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_serde_get_generators_for_event_source_requests(value in pr_strats::get_generators_for_event_source_requests()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_serde_get_generators_for_event_source_responses(value in pr_strats::get_generators_for_event_source_responses()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_serde_get_plugin_requests(value in pr_strats::get_plugin_requests()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_serde_get_plugin_responses(value in pr_strats::get_plugin_responses()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_serde_get_plugin_deployment_requests(
            plugin_deployment_request in pr_strats::get_plugin_deployment_requests()
        ) {
            check_encode_decode_invariant(plugin_deployment_request)
        }

        #[test]
        fn test_serde_get_plugin_deployment_responses(
            plugin_deployment_response in pr_strats::get_plugin_deployment_responses()
        ) {
            check_encode_decode_invariant(plugin_deployment_response)
        }

        #[test]
        fn test_serde_tear_down_plugin_requests(value in pr_strats::tear_down_plugin_requests()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_serde_tear_down_plugin_responses(value in pr_strats::tear_down_plugin_responses()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_serde_get_plugin_health_requests(
            value in pr_strats::get_plugin_health_requests()
        ) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_serde_get_plugin_health_responses(
            value in pr_strats::get_plugin_health_responses()
        ) {
            check_encode_decode_invariant(value)
        }

    }
}

mod plugin_sdk_generators {
    use strategies::plugin_sdk_generators as gen_strats;

    use super::*;

    proptest! {
        #[test]
        fn test_serde_run_generator_requests(value in gen_strats::run_generator_requests()) {
            if value.data.is_empty() {
                expect_serde_error(value);
            } else {
                check_encode_decode_invariant(value)
            }
        }

        fn test_serde_run_generator_responses(value in gen_strats::run_generator_responses()) {
            check_encode_decode_invariant(value)
        }
    }
}

mod plugin_work_queue {
    use strategies::plugin_work_queue as pwq_strats;

    use super::*;

    proptest! {
        #[test]
        fn test_acknowledge_generator_requests(value in pwq_strats::acknowledge_generator_requests()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_acknowledge_generator_responses(value in pwq_strats::acknowledge_generator_responses()) {
            check_encode_decode_invariant(value)
        }


        #[test]
        fn test_acknowledge_analyzer_requests(value in pwq_strats::acknowledge_analyzer_requests()) {
            check_encode_decode_invariant(value)
        }


        #[test]
        fn test_acknowledge_analyzer_responses(value in pwq_strats::acknowledge_analyzer_responses()) {
            check_encode_decode_invariant(value)
        }


        #[test]
        fn test_get_execute_analyzer_requests(value in pwq_strats::get_execute_analyzer_requests()) {
            check_encode_decode_invariant(value)
        }


        #[test]
        fn test_get_execute_analyzer_responses(
            value in pwq_strats::get_execute_analyzer_responses()
        ) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_get_execute_generator_requests(value in pwq_strats::get_execute_generator_requests()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_get_execute_generator_responses(
            value in pwq_strats::get_execute_generator_responses()
        ) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_push_execute_analyzer_requests(value in pwq_strats::push_execute_analyzer_requests()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_push_execute_analyzer_responses(value in pwq_strats::push_execute_analyzer_responses()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_push_execute_generator_requests(value in pwq_strats::push_execute_generator_requests()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_push_execute_generator_responses(value in pwq_strats::push_execute_generator_responses()) {
            check_encode_decode_invariant(value)
        }
    }
}

mod graph_schema_manager {
    use strategies::graph_schema_manager as sm_strats;

    use super::*;

    proptest! {
        #[test]
        fn test_deploy_schema_requests(value in sm_strats::deploy_schema_requests()) {
            check_encode_decode_invariant(value)
        }

        #[test]
        fn test_get_edge_schema_responses(value in sm_strats::get_edge_schema_responses()) {
            check_encode_decode_invariant(value)
        }
    }
}

mod analyzer_sdk {

    use strategies::analyzer_sdk as as_strats;

    use super::*;

    proptest! {
        #[test]
        fn test_run_analyzer_request(value in as_strats::run_analyzer_requests()) {
            check_encode_decode_invariant(value)
        }
    }
}
