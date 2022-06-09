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

pub fn vec_of_uuids() -> impl Strategy<Value = Vec<Uuid>> {
    proptest::collection::vec(uuids(), 10)
}

pub fn string_not_empty() -> proptest::string::RegexGeneratorStrategy<String> {
    proptest::string::string_regex(".+").expect("Invalid regex")
}

pub fn vec_not_empty<T>() -> impl Strategy<Value = Vec<T>>
where
    T: Arbitrary,
{
    any::<Vec<T>>().prop_filter("Only accept non-empty vecs", |v| !v.is_empty())
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

pub mod graph {
    use proptest::collection;
    use rust_proto_new::graplinc::grapl::api::graph::v1beta1::{
        DecrementOnlyIntProp,
        DecrementOnlyUintProp,
        Edge,
        EdgeList,
        GraphDescription,
        IdStrategy,
        IdentifiedGraph,
        IdentifiedNode,
        ImmutableIntProp,
        ImmutableStrProp,
        ImmutableUintProp,
        IncrementOnlyIntProp,
        IncrementOnlyUintProp,
        MergedEdge,
        MergedEdgeList,
        MergedGraph,
        MergedNode,
        NodeDescription,
        NodeProperty,
        Property,
        Session,
        Static,
        Strategy as GraphStrategy,
    };

    use super::*;

    //
    // DecrementOnlyIntProp
    //

    prop_compose! {
        pub fn decrement_only_int_props()(
            prop in any::<i64>(),
        ) -> DecrementOnlyIntProp {
            DecrementOnlyIntProp {
                prop
            }
        }
    }

    //
    // DecrementOnlyUintProp
    //

    prop_compose! {
        pub fn decrement_only_uint_props()(
            prop in any::<u64>(),
        ) -> DecrementOnlyUintProp {
            DecrementOnlyUintProp {
                prop
            }
        }
    }

    //
    // ImmutableIntProp
    //

    prop_compose! {
        pub fn immutable_int_props()(
            prop in any::<i64>(),
        ) -> ImmutableIntProp {
            ImmutableIntProp {
                prop
            }
        }
    }

    //
    // ImmutableStrProp
    //

    prop_compose! {
        pub fn immutable_str_props()(
            prop in any::<String>(),
        ) -> ImmutableStrProp {
            ImmutableStrProp {
                prop
            }
        }
    }

    //
    // ImmutableUintProp
    //

    prop_compose! {
        pub fn immutable_uint_props()(
            prop in any::<u64>(),
        ) -> ImmutableUintProp {
            ImmutableUintProp {
                prop
            }
        }
    }

    //
    // IncrementOnlyIntProp
    //

    prop_compose! {
        pub fn increment_only_int_props()(
            prop in any::<i64>(),
        ) -> IncrementOnlyIntProp {
            IncrementOnlyIntProp {
                prop
            }
        }
    }

    //
    // IncrementOnlyUintProp
    //

    prop_compose! {
        pub fn increment_only_uint_props()(
            prop in any::<u64>(),
        ) -> IncrementOnlyUintProp {
            IncrementOnlyUintProp {
                prop
            }
        }
    }

    //
    // Edge
    //

    prop_compose! {
        pub fn edges()(
            to_node_key in any::<String>(),
            from_node_key in any::<String>(),
            edge_name in any::<String>(),
        ) -> Edge {
            Edge {
                to_node_key,
                from_node_key,
                edge_name,
            }
        }
    }

    //
    // EdgeList
    //

    prop_compose! {
        pub fn edge_lists()(
            edges in collection::vec(edges(), 10),
        ) -> EdgeList {
            EdgeList {
                edges
            }
        }
    }

    //
    // Session
    //

    prop_compose! {
        pub fn sessions()(
            primary_key_properties in collection::vec(any::<String>(), 10),
            primary_key_requires_asset_id in any::<bool>(),
            create_time in any::<u64>(),
            last_seen_time in any::<u64>(),
            terminate_time in any::<u64>(),
        ) -> Session {
            Session {
                primary_key_properties,
                primary_key_requires_asset_id,
                create_time,
                last_seen_time,
                terminate_time
            }
        }
    }

    //
    // Static
    //

    prop_compose! {
        pub fn statics()(
            primary_key_properties in collection::vec(any::<String>(), 10),
            primary_key_requires_asset_id in any::<bool>(),
        ) -> Static {
            Static {
                primary_key_properties,
                primary_key_requires_asset_id
            }
        }
    }

    //
    // Strategy
    //

    pub fn strategies() -> impl Strategy<Value = GraphStrategy> {
        prop_oneof![
            sessions().prop_map(GraphStrategy::Session),
            statics().prop_map(GraphStrategy::Static),
        ]
    }

    //
    // IdStrategy
    //

    prop_compose! {
        pub fn id_strategies()(
            strategy in strategies()
        ) -> IdStrategy {
            IdStrategy { strategy }
        }
    }

    //
    // Property
    //

    pub fn properties() -> impl Strategy<Value = Property> {
        prop_oneof![
            decrement_only_int_props().prop_map(Property::DecrementOnlyIntProp),
            decrement_only_uint_props().prop_map(Property::DecrementOnlyUintProp),
            immutable_int_props().prop_map(Property::ImmutableIntProp),
            immutable_str_props().prop_map(Property::ImmutableStrProp),
            immutable_uint_props().prop_map(Property::ImmutableUintProp),
            increment_only_int_props().prop_map(Property::IncrementOnlyIntProp),
            increment_only_uint_props().prop_map(Property::IncrementOnlyUintProp),
        ]
    }

    //
    // NodeProperty
    //

    prop_compose! {
        pub fn node_properties()(
            property in properties()
        ) -> NodeProperty {
            NodeProperty { property }
        }
    }

    //
    // NodeDescription
    //

    prop_compose! {
        pub fn node_descriptions()(
            properties in collection::hash_map(any::<String>(), node_properties(), 10),
            node_key in any::<String>(),
            node_type in any::<String>(),
            id_strategy in collection::vec(id_strategies(), 10),
        ) -> NodeDescription {
            NodeDescription {
                properties,
                node_key,
                node_type,
                id_strategy
            }
        }
    }

    //
    // GraphDescription
    //

    prop_compose! {
        pub fn graph_descriptions()(
            nodes in collection::hash_map(any::<String>(), node_descriptions(), 10),
            edges in collection::hash_map(any::<String>(), edge_lists(), 10),
        ) -> GraphDescription {
            GraphDescription {
                nodes,
                edges,
            }
        }
    }

    //
    // IdentifiedNode
    //

    prop_compose! {
        pub fn identified_nodes()(
            properties in collection::hash_map(any::<String>(), node_properties(), 10),
            node_key in any::<String>(),
            node_type in any::<String>(),
        ) -> IdentifiedNode {
            IdentifiedNode {
                properties,
                node_key,
                node_type
            }
        }
    }

    //
    // IdentifiedGraph
    //

    prop_compose! {
        pub fn identified_graphs()(
            nodes in collection::hash_map(any::<String>(), identified_nodes(), 10),
            edges in collection::hash_map(any::<String>(), edge_lists(), 10),
        ) -> IdentifiedGraph {
            IdentifiedGraph {
                nodes,
                edges
            }
        }
    }

    //
    // MergedEdge
    //

    prop_compose! {
        pub fn merged_edges()(
            from_uid in any::<String>(),
            from_node_key in any::<String>(),
            to_uid in any::<String>(),
            to_node_key in any::<String>(),
            edge_name in any::<String>(),
        ) -> MergedEdge {
            MergedEdge {
                from_uid,
                from_node_key,
                to_uid,
                to_node_key,
                edge_name
            }
        }
    }

    //
    // MergedEdgeList
    //

    prop_compose! {
        pub fn merged_edge_lists()(
            edges in collection::vec(merged_edges(), 10),
        ) -> MergedEdgeList {
            MergedEdgeList { edges }
        }
    }

    //
    // MergedNode
    //

    prop_compose! {
        pub fn merged_nodes()(
            properties in collection::hash_map(any::<String>(), node_properties(), 10),
            uid in any::<u64>(),
            node_key in any::<String>(),
            node_type in any::<String>(),
        ) -> MergedNode {
            MergedNode {
                properties,
                uid,
                node_key,
                node_type
            }
        }
    }

    //
    // MergedGraph
    //

    prop_compose! {
        pub fn merged_graphs()(
            nodes in collection::hash_map(any::<String>(), merged_nodes(), 10),
            edges in collection::hash_map(any::<String>(), merged_edge_lists(), 10),
        ) -> MergedGraph {
            MergedGraph {
                nodes,
                edges,
            }
        }
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

pub mod plugin_registry {
    use rust_proto_new::graplinc::grapl::api::plugin_registry::v1beta1::{
        CreatePluginRequest,
        CreatePluginRequestMetadata,
        CreatePluginResponse,
        DeployPluginRequest,
        DeployPluginResponse,
        GetAnalyzersForTenantRequest,
        GetAnalyzersForTenantResponse,
        GetGeneratorsForEventSourceRequest,
        GetGeneratorsForEventSourceResponse,
        GetPluginRequest,
        GetPluginResponse,
        Plugin,
        PluginType,
        TearDownPluginRequest,
        TearDownPluginResponse,
    };

    use super::*;

    pub fn plugin_types() -> BoxedStrategy<PluginType> {
        prop_oneof![
            // For cases without data, `Just` is all you need
            Just(PluginType::Generator),
            Just(PluginType::Analyzer),
        ]
        .boxed()
    }

    prop_compose! {
        pub fn plugins()(
            plugin_id in uuids(),
            display_name in string_not_empty(),
            plugin_type in plugin_types(),
            plugin_binary in any::<Vec<u8>>(),
        ) -> Plugin {
            Plugin {
                plugin_id,
                display_name,
                plugin_type,
                plugin_binary,
            }
        }
    }

    pub fn create_plugin_requests() -> impl Strategy<Value = CreatePluginRequest> {
        prop_oneof![
            any::<Vec<u8>>().prop_map(CreatePluginRequest::Chunk),
            create_plugin_request_metadatas().prop_map(CreatePluginRequest::Metadata)
        ]
    }

    prop_compose! {
        pub fn create_plugin_request_metadatas()(
            tenant_id in uuids(),
            display_name in string_not_empty(),
            plugin_type in plugin_types(),
        ) -> CreatePluginRequestMetadata {
            CreatePluginRequestMetadata {
                tenant_id,
                display_name,
                plugin_type,
            }
        }
    }

    prop_compose! {
        pub fn create_plugin_responses()(
            plugin_id in uuids(),
        ) -> CreatePluginResponse {
            CreatePluginResponse{
                plugin_id,
            }
        }
    }

    prop_compose! {
        pub fn get_analyzers_for_tenant_requests()(
            tenant_id in uuids(),
        ) -> GetAnalyzersForTenantRequest {
            GetAnalyzersForTenantRequest{
                tenant_id,
            }
        }
    }
    prop_compose! {
        pub fn get_analyzers_for_tenant_responses()(
            plugin_ids in vec_of_uuids()
        ) -> GetAnalyzersForTenantResponse {
            GetAnalyzersForTenantResponse{
                plugin_ids
            }
        }
    }

    prop_compose! {
        pub fn deploy_plugin_requests()(
            plugin_id in uuids()
        ) -> DeployPluginRequest {
            DeployPluginRequest{
                plugin_id
            }
        }
    }

    pub fn deploy_plugin_responses() -> impl Strategy<Value = DeployPluginResponse> {
        Just(DeployPluginResponse {})
    }

    prop_compose! {
        pub fn get_generators_for_event_source_requests()(
            event_source_id in uuids()
        ) -> GetGeneratorsForEventSourceRequest {
            GetGeneratorsForEventSourceRequest{
                event_source_id
            }
        }
    }

    prop_compose! {
        pub fn get_generators_for_event_source_responses()(
            plugin_ids in vec_of_uuids()
        ) -> GetGeneratorsForEventSourceResponse {
            GetGeneratorsForEventSourceResponse {
                plugin_ids
            }
        }
    }

    prop_compose! {
        pub fn get_plugin_requests()(
            plugin_id in uuids(),
            tenant_id in uuids(),
        ) -> GetPluginRequest {
            GetPluginRequest {
                plugin_id,
                tenant_id,
            }
        }
    }

    prop_compose! {
        pub fn get_plugin_responses()(
            plugin in plugins(),
        ) -> GetPluginResponse {
            GetPluginResponse {
                plugin
            }
        }
    }

    prop_compose! {
        pub fn tear_down_plugin_requests()(
            plugin_id in uuids()
        ) -> TearDownPluginRequest {
            TearDownPluginRequest{
                plugin_id
            }
        }
    }

    pub fn tear_down_plugin_responses() -> impl Strategy<Value = TearDownPluginResponse> {
        Just(TearDownPluginResponse {})
    }
}

pub mod plugin_sdk_generators {
    use rust_proto_new::graplinc::grapl::api::plugin_sdk::generators::v1beta1 as native;

    use super::*;

    prop_compose! {
        fn generated_graphs()(
            graph_description in graph::graph_descriptions()
        ) -> native::GeneratedGraph {
            native::GeneratedGraph {
                graph_description
            }
        }
    }

    prop_compose! {
        pub fn run_generator_requests()(
            data in any::<Vec<u8>>()
        ) -> native::RunGeneratorRequest {
            native::RunGeneratorRequest {
                data
            }
        }
    }

    prop_compose! {
        pub fn run_generator_responses()(
            generated_graph in generated_graphs(),
        ) -> native::RunGeneratorResponse {
            native::RunGeneratorResponse {
                generated_graph
            }
        }
    }
}

pub mod plugin_work_queue {
    use rust_proto_new::graplinc::grapl::api::plugin_work_queue::v1beta1 as native;

    use super::*;

    prop_compose! {
        pub fn execution_jobs()(
            tenant_id in uuids(),
            plugin_id in uuids(),
            data in vec_not_empty::<u8>(),
        ) -> native::ExecutionJob {
            native::ExecutionJob {
                tenant_id,
                plugin_id,
                data,
            }
        }
    }

    prop_compose! {
        pub fn acknowledge_generator_requests()(
            request_id in any::<i64>(),
            success in any::<bool>(),
        ) -> native::AcknowledgeGeneratorRequest {
            native::AcknowledgeGeneratorRequest {
                request_id,
                success,
            }
        }
    }

    pub fn acknowledge_generator_responses(
    ) -> impl Strategy<Value = native::AcknowledgeGeneratorResponse> {
        Just(native::AcknowledgeGeneratorResponse {})
    }

    prop_compose! {
        pub fn acknowledge_analyzer_requests()(
            request_id in any::<i64>(),
            success in any::<bool>(),
        ) -> native::AcknowledgeAnalyzerRequest {
            native::AcknowledgeAnalyzerRequest {
                request_id,
                success,
            }
        }
    }

    pub fn acknowledge_analyzer_responses(
    ) -> impl Strategy<Value = native::AcknowledgeAnalyzerResponse> {
        Just(native::AcknowledgeAnalyzerResponse {})
    }

    pub fn maybe_jobs() -> impl Strategy<Value = Option<native::ExecutionJob>> {
        proptest::option::of(execution_jobs())
    }

    pub fn get_execute_analyzer_requests(
    ) -> impl Strategy<Value = native::GetExecuteAnalyzerRequest> {
        Just(native::GetExecuteAnalyzerRequest {})
    }

    prop_compose! {
        pub fn get_execute_analyzer_responses()(
            execution_job in maybe_jobs(),
            request_id in any::<i64>(),
        ) -> native::GetExecuteAnalyzerResponse {
            native::GetExecuteAnalyzerResponse {
                execution_job,
                request_id,
            }
        }
    }

    pub fn get_execute_generator_requests(
    ) -> impl Strategy<Value = native::GetExecuteGeneratorRequest> {
        Just(native::GetExecuteGeneratorRequest {})
    }

    prop_compose! {
        pub fn get_execute_generator_responses()(
            execution_job in maybe_jobs(),
            request_id in any::<i64>(),
        ) -> native::GetExecuteGeneratorResponse {
            native::GetExecuteGeneratorResponse {
                execution_job,
                request_id,
            }
        }
    }

    prop_compose! {
        pub fn put_execute_analyzer_requests()(
            execution_job in execution_jobs(),
        ) -> native::PutExecuteAnalyzerRequest {
            native::PutExecuteAnalyzerRequest {
                execution_job,
            }
        }
    }

    pub fn put_execute_analyzer_responses(
    ) -> impl Strategy<Value = native::PutExecuteAnalyzerResponse> {
        Just(native::PutExecuteAnalyzerResponse {})
    }

    prop_compose! {
        pub fn put_execute_generator_requests()(
            execution_job in execution_jobs(),
        ) -> native::PutExecuteGeneratorRequest {
            native::PutExecuteGeneratorRequest {
                execution_job,
            }
        }
    }

    pub fn put_execute_generator_responses(
    ) -> impl Strategy<Value = native::PutExecuteGeneratorResponse> {
        Just(native::PutExecuteGeneratorResponse {})
    }
}

pub mod lens_manager {
    use rust_proto_new::graplinc::grapl::api::lens_manager::v1beta1::messages as native;

    use super::*;

    prop_compose! {
        pub fn create_lens_request()(
            tenant_id in uuids(),
            lens_type in string_not_empty(),
            lens_name in string_not_empty(),
            is_engagement in any::<bool>(),
        ) -> native::CreateLensRequest {
            native::CreateLensRequest {
                tenant_id,
                lens_type,
                lens_name,
                is_engagement,
            }
        }
    }

    prop_compose! {
        pub fn create_lens_response()(
            lens_uid in any::<u64>(),
        ) -> native::CreateLensResponse {
            native::CreateLensResponse {
                lens_uid,
            }
        }
    }

    pub fn merge_behavior() -> impl Strategy<Value = native::MergeBehavior> {
        prop_oneof![
            Just(native::MergeBehavior::Preserve),
            Just(native::MergeBehavior::Close),
        ]
    }

    prop_compose! {
        pub fn merge_lens_request()(
            tenant_id in uuids(),
            source_lens_uid in any::<u64>(),
            target_lens_uid in any::<u64>(),
            merge_behavior in merge_behavior(),
        ) -> native::MergeLensRequest {
            native::MergeLensRequest {
                tenant_id,
                source_lens_uid,
                target_lens_uid,
                merge_behavior,
            }
        }
    }

    pub fn merge_lens_response() -> impl Strategy<Value = native::MergeLensResponse> {
        Just(native::MergeLensResponse {})
    }

    prop_compose! {
        pub fn close_lens_request()(
            tenant_id in uuids(),
            lens_uid in any::<u64>(),
        ) -> native::CloseLensRequest {
            native::CloseLensRequest {
                tenant_id,
                lens_uid,
            }
        }
    }

    pub fn close_lens_response() -> impl Strategy<Value = native::CloseLensResponse> {
        Just(native::CloseLensResponse {})
    }

    prop_compose! {
        pub fn add_node_to_scope_request()(
            tenant_id in uuids(),
            lens_uid in any::<u64>(),
            uid in any::<u64>(),
        ) -> native::AddNodeToScopeRequest {
            native::AddNodeToScopeRequest {
                tenant_id,
                lens_uid,
                uid,
            }
        }
    }

    pub fn add_node_to_scope_response() -> impl Strategy<Value = native::AddNodeToScopeResponse> {
        Just(native::AddNodeToScopeResponse {})
    }

    prop_compose! {
        pub fn remove_node_from_scope_request()(
            tenant_id in uuids(),
            lens_uid in any::<u64>(),
            uid in any::<u64>(),
        ) -> native::RemoveNodeFromScopeRequest {
            native::RemoveNodeFromScopeRequest {
                tenant_id,
                lens_uid,
                uid,
            }
        }
    }

    pub fn remove_node_from_scope_response(
    ) -> impl Strategy<Value = native::RemoveNodeFromScopeResponse> {
        Just(native::RemoveNodeFromScopeResponse {})
    }

    prop_compose! {
        pub fn remove_node_from_all_scopes_request()(
            tenant_id in uuids(),
            uid in any::<u64>(),
        ) -> native::RemoveNodeFromAllScopesRequest {
            native::RemoveNodeFromAllScopesRequest {
                tenant_id,
                uid,
            }
        }
    }

    pub fn remove_node_from_all_scopes_response(
    ) -> impl Strategy<Value = native::RemoveNodeFromAllScopesResponse> {
        Just(native::RemoveNodeFromAllScopesResponse {})
    }
}
