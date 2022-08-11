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

pub mod pipeline {
    use std::fmt::Debug;

    use rust_proto::{
        graplinc::grapl::pipeline::v1beta1::{
            Envelope,
            RawLog,
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
            RawLog::new(log_event)
        }
    }

    //
    // Envelope
    //

    pub fn envelopes<T>(
        inner_strategy: impl Strategy<Value = T>,
    ) -> impl Strategy<Value = Envelope<T>>
    where
        T: SerDe + Debug,
    {
        (uuids(), uuids(), uuids(), inner_strategy).prop_map(
            |(tenant_id, trace_id, event_source_id, inner_message)| -> Envelope<T> {
                Envelope::new(tenant_id, trace_id, event_source_id, inner_message)
            },
        )
    }
}

pub mod graph {
    use proptest::collection;
    use rust_proto::graplinc::grapl::api::graph::v1beta1::{
        DecrementOnlyIntProp,
        DecrementOnlyUintProp,
        Edge,
        EdgeList,
        ExecutionHit,
        GraphDescription,
        IdStrategy,
        IdentifiedGraph,
        IdentifiedNode,
        ImmutableIntProp,
        ImmutableStrProp,
        ImmutableUintProp,
        IncrementOnlyIntProp,
        IncrementOnlyUintProp,
        Lens,
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
    // Lens
    //

    prop_compose! {
        pub fn lenses()(
            lens_type in any::<String>(),
            lens_name in any::<String>(),
            uid in any::<u64>(),
            score in any::<u64>(),
        ) -> Lens {
            Lens {
                lens_type,
                lens_name,
                uid: Some(uid),
                score: Some(score)
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
    // ExecutionHit
    //

    prop_compose! {
        pub fn execution_hits()(
            nodes in collection::hash_map(any::<String>(), merged_nodes(), 10),
            edges in collection::hash_map(any::<String>(), merged_edge_lists(), 10),
            analyzer_name in any::<String>(),
            risk_score in any::<u64>(),
            lenses in collection::vec(lenses(), 10),
            risky_node_keys in collection::vec(any::<String>(), 10)
        ) -> ExecutionHit {
            ExecutionHit{
                nodes,
                edges,
                analyzer_name,
                risk_score,
                lenses,
                risky_node_keys
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
    use rust_proto::graplinc::grapl::api::pipeline_ingress::v1beta1::{
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
            PublishRawLogRequest::new(
                event_source_id,
                tenant_id,
                log_event
            )
        }
    }

    //
    // PublishRawLogResponse
    //

    prop_compose! {
        pub fn publish_raw_log_responses()(
            created_time in any::<SystemTime>(),
        ) -> PublishRawLogResponse {
            PublishRawLogResponse::new(
                created_time,
            )
        }
    }
}

pub mod event_source {
    use rust_proto::graplinc::grapl::api::event_source::v1beta1 as native;

    use super::*;

    prop_compose! {
        pub fn create_event_source_requests()(
            display_name in string_not_empty(),
            description in string_not_empty(),
            tenant_id in uuids(),
        ) -> native::CreateEventSourceRequest {
            native::CreateEventSourceRequest {
                display_name,
                description,
                tenant_id,
            }
        }
    }

    prop_compose! {
        pub fn create_event_source_responses()(
            event_source_id in uuids(),
            created_time in any::<SystemTime>(),
        ) -> native::CreateEventSourceResponse {
            native::CreateEventSourceResponse {
                event_source_id,
                created_time,
            }
        }
    }

    prop_compose! {
        pub fn update_event_source_requests()(
            event_source_id in uuids(),
            display_name in string_not_empty(),
            description in string_not_empty(),
            active in any::<bool>(),
        ) -> native::UpdateEventSourceRequest {
            native::UpdateEventSourceRequest {
                event_source_id,
                display_name,
                description,
                active,
            }
        }
    }

    prop_compose! {
        pub fn update_event_source_responses()(
            event_source_id in uuids(),
            last_updated_time in any::<SystemTime>(),
        ) -> native::UpdateEventSourceResponse {
            native::UpdateEventSourceResponse {
                event_source_id,
                last_updated_time,
            }
        }
    }

    prop_compose! {
        pub fn event_sources()(
            tenant_id in uuids(),
            event_source_id in uuids(),
            display_name in string_not_empty(),
            description in string_not_empty(),
            created_time in any::<SystemTime>(),
            last_updated_time in any::<SystemTime>(),
            active in any::<bool>(),
        ) -> native::EventSource {
            native::EventSource {
                tenant_id,
                event_source_id,
                display_name,
                description,
                created_time,
                last_updated_time,
                active,
            }
        }
    }

    prop_compose! {
        pub fn get_event_source_requests()(
            event_source_id in uuids(),
        ) -> native::GetEventSourceRequest {
            native::GetEventSourceRequest {
                event_source_id,
            }
        }
    }

    prop_compose! {
        pub fn get_event_source_responses()(
            event_source in event_sources(),
        ) -> native::GetEventSourceResponse {
            native::GetEventSourceResponse {
                event_source
            }
        }
    }
}

pub mod plugin_registry {
    use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
        CreatePluginRequest,
        CreatePluginResponse,
        DeployPluginRequest,
        DeployPluginResponse,
        GetAnalyzersForTenantRequest,
        GetAnalyzersForTenantResponse,
        GetGeneratorsForEventSourceRequest,
        GetGeneratorsForEventSourceResponse,
        GetPluginHealthRequest,
        GetPluginHealthResponse,
        GetPluginRequest,
        GetPluginResponse,
        PluginHealthStatus,
        PluginMetadata,
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
        pub fn plugin_metadatas()(
            tenant_id in uuids(),
            display_name in string_not_empty(),
            plugin_type in plugin_types(),
            event_source_id in uuids(),
        ) -> PluginMetadata {
            match plugin_type {
                PluginType::Generator => PluginMetadata::new(
                    tenant_id,
                    display_name,
                    plugin_type,
                    Some(event_source_id),
                ),
                _ => PluginMetadata::new(
                    tenant_id,
                    display_name,
                    plugin_type,
                    None,
                )
            }
        }
    }

    pub fn create_plugin_requests() -> impl Strategy<Value = CreatePluginRequest> {
        prop_oneof![
            bytes(1024).prop_map(CreatePluginRequest::Chunk),
            plugin_metadatas().prop_map(CreatePluginRequest::Metadata)
        ]
    }

    prop_compose! {
        pub fn create_plugin_responses()(
            plugin_id in uuids(),
        ) -> CreatePluginResponse {
            CreatePluginResponse::new(plugin_id)
        }
    }

    prop_compose! {
        pub fn get_analyzers_for_tenant_requests()(
            tenant_id in uuids(),
        ) -> GetAnalyzersForTenantRequest {
            GetAnalyzersForTenantRequest::new(tenant_id)
        }
    }
    prop_compose! {
        pub fn get_analyzers_for_tenant_responses()(
            plugin_ids in vec_of_uuids()
        ) -> GetAnalyzersForTenantResponse {
            GetAnalyzersForTenantResponse::new(plugin_ids)
        }
    }

    prop_compose! {
        pub fn deploy_plugin_requests()(
            plugin_id in uuids()
        ) -> DeployPluginRequest {
            DeployPluginRequest::new(plugin_id)
        }
    }

    pub fn deploy_plugin_responses() -> impl Strategy<Value = DeployPluginResponse> {
        Just(DeployPluginResponse {})
    }

    prop_compose! {
        pub fn get_generators_for_event_source_requests()(
            event_source_id in uuids()
        ) -> GetGeneratorsForEventSourceRequest {
            GetGeneratorsForEventSourceRequest::new(event_source_id)
        }
    }

    prop_compose! {
        pub fn get_generators_for_event_source_responses()(
            plugin_ids in vec_of_uuids()
        ) -> GetGeneratorsForEventSourceResponse {
            GetGeneratorsForEventSourceResponse::new(plugin_ids)
        }
    }

    prop_compose! {
        pub fn get_plugin_requests()(
            plugin_id in uuids(),
            tenant_id in uuids(),
        ) -> GetPluginRequest {
            GetPluginRequest::new(plugin_id, tenant_id)
        }
    }

    prop_compose! {
        pub fn get_plugin_responses()(
            plugin_id in uuids(),
            plugin_metadata in plugin_metadatas(),
        ) -> GetPluginResponse {
            GetPluginResponse::new(plugin_id, plugin_metadata)
        }
    }

    prop_compose! {
        pub fn tear_down_plugin_requests()(
            plugin_id in uuids()
        ) -> TearDownPluginRequest {
            TearDownPluginRequest::new(plugin_id)
        }
    }

    pub fn tear_down_plugin_responses() -> impl Strategy<Value = TearDownPluginResponse> {
        Just(TearDownPluginResponse {})
    }

    prop_compose! {
        pub fn get_plugin_health_requests()(
            plugin_id in uuids()
        ) -> GetPluginHealthRequest {
            GetPluginHealthRequest::new(plugin_id)
        }
    }

    pub fn plugin_health_statuses() -> BoxedStrategy<PluginHealthStatus> {
        prop_oneof![
            // For cases without data, `Just` is all you need
            Just(PluginHealthStatus::NotDeployed),
            Just(PluginHealthStatus::Pending),
            Just(PluginHealthStatus::Running),
            Just(PluginHealthStatus::Dead),
        ]
        .boxed()
    }

    prop_compose! {
        pub fn get_plugin_health_responses()(
            health_status in plugin_health_statuses()
        ) -> GetPluginHealthResponse{
            GetPluginHealthResponse::new(health_status)
        }
    }
}

pub mod plugin_sdk_generators {
    use rust_proto::graplinc::grapl::api::plugin_sdk::generators::v1beta1 as native;

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
            data in bytes(1024)
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
    use rust_proto::graplinc::grapl::api::plugin_work_queue::v1beta1 as native;

    use super::*;

    prop_compose! {
        pub fn execution_jobs()(
            data in bytes(1024),
            tenant_id in uuids(),
            trace_id in uuids(),
            event_source_id in uuids(),
        ) -> native::ExecutionJob {
            native::ExecutionJob::new(data, tenant_id, trace_id, event_source_id)
        }
    }

    prop_compose! {
        pub fn acknowledge_generator_requests()(
            request_id in any::<i64>(),
            graph_description in proptest::option::of(graph::graph_descriptions()),
            plugin_id in uuids(),
            tenant_id in uuids(),
            trace_id in uuids(),
            event_source_id in uuids(),
        ) -> native::AcknowledgeGeneratorRequest {
            native::AcknowledgeGeneratorRequest::new(
                request_id,
                graph_description,
                plugin_id,
                tenant_id,
                trace_id,
                event_source_id,
            )
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
            plugin_id in uuids(),
        ) -> native::AcknowledgeAnalyzerRequest {
            native::AcknowledgeAnalyzerRequest {
                request_id,
                success,
                plugin_id,
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

    prop_compose! {
        pub fn get_execute_analyzer_requests()(
            plugin_id in uuids(),
        ) -> native::GetExecuteAnalyzerRequest {
            native::GetExecuteAnalyzerRequest {
                plugin_id,
            }
        }
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

    prop_compose! {
        pub fn get_execute_generator_requests()(
            plugin_id in uuids(),
        ) -> native::GetExecuteGeneratorRequest {
            native::GetExecuteGeneratorRequest {
                plugin_id,
            }
        }
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
        pub fn push_execute_analyzer_requests()(
            execution_job in execution_jobs(),
            plugin_id in uuids(),
        ) -> native::PushExecuteAnalyzerRequest {
            native::PushExecuteAnalyzerRequest::new(
                execution_job,
                plugin_id,
            )
        }
    }

    pub fn push_execute_analyzer_responses(
    ) -> impl Strategy<Value = native::PushExecuteAnalyzerResponse> {
        Just(native::PushExecuteAnalyzerResponse {})
    }

    prop_compose! {
        pub fn push_execute_generator_requests()(
            execution_job in execution_jobs(),
            plugin_id in uuids(),
        ) -> native::PushExecuteGeneratorRequest {
            native::PushExecuteGeneratorRequest::new(
                execution_job,
                plugin_id,
            )
        }
    }

    pub fn push_execute_generator_responses(
    ) -> impl Strategy<Value = native::PushExecuteGeneratorResponse> {
        Just(native::PushExecuteGeneratorResponse {})
    }
}
