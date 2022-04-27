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
