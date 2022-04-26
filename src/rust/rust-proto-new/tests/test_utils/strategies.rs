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

pub mod plugin_registry {
    use super::*;
    use rust_proto_new::graplinc::grapl::api::plugin_registry::v1beta1::{
        PluginType, Plugin, CreatePluginRequest, CreatePluginResponse
    };

    pub fn plugin_types() -> BoxedStrategy<PluginType> {
        prop_oneof![
            // For cases without data, `Just` is all you need
            Just(PluginType::Generator),
            Just(PluginType::Analyzer),
        ].boxed()
    }

    prop_compose! {
        pub fn plugins()(
            plugin_id in uuids(),
            display_name in any::<String>(),
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
            plugin_artifact in any::<Vec<u8>>(),
            tenant_id in uuids(),
            display_name in any::<String>(),
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
}