use rust_proto::graplinc::grapl::api::event_source::v1beta1::client::EventSourceServiceClient;

use crate::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug)]
pub struct EventSourceClientConfig {
    #[clap(long, env)]
    pub event_source_client_address: String,
    #[clap(long, env, default_value = "500")]
    pub event_source_healthcheck_polling_interval_ms: u64,
}

impl From<EventSourceClientConfig> for GenericGrpcClientConfig {
    fn from(val: EventSourceClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.event_source_client_address,
            healthcheck_polling_interval_ms: val.event_source_healthcheck_polling_interval_ms,
        }
    }
}

impl GrpcClientConfig for EventSourceClientConfig {
    type Client = EventSourceServiceClient;
}
