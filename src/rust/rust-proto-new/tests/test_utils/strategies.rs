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
            sessions().prop_map(|session| GraphStrategy::Session(session)),
            statics().prop_map(|static_| GraphStrategy::Static(static_)),
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
            decrement_only_int_props().prop_map(|p| Property::DecrementOnlyIntProp(p)),
            decrement_only_uint_props().prop_map(|p| Property::DecrementOnlyUintProp(p)),
            immutable_int_props().prop_map(|p| Property::ImmutableIntProp(p)),
            immutable_str_props().prop_map(|p| Property::ImmutableStrProp(p)),
            immutable_uint_props().prop_map(|p| Property::ImmutableUintProp(p)),
            increment_only_int_props().prop_map(|p| Property::IncrementOnlyIntProp(p)),
            increment_only_uint_props().prop_map(|p| Property::IncrementOnlyUintProp(p)),
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

    prop_compose! {
        pub fn create_plugin_requests()(
            plugin_artifact in string_not_empty().prop_map(|s| s.as_bytes().to_vec()),
            tenant_id in uuids(),
            display_name in string_not_empty(),
            plugin_type in plugin_types(),
        ) -> CreatePluginRequest {
            CreatePluginRequest {
                plugin_artifact,
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
