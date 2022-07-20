use std::net::SocketAddr;

use rust_proto::graplinc::grapl::api::event_source::v1beta1::client::EventSourceServiceClient;

use crate::grpc_client_config::GrpcClientConfig;

#[derive(clap::Parser, Debug)]
pub struct EventSourceClientConfig {
    #[clap(long, env)]
    pub event_source_client_address: SocketAddr,
    #[clap(long, env, default_value = "500")]
    pub event_source_healthcheck_polling_interval_ms: u64,
}

impl GrpcClientConfig for EventSourceClientConfig {
    type Client = EventSourceServiceClient;

    fn address(&self) -> SocketAddr {
        self.event_source_client_address
    }
    fn healthcheck_polling_interval_ms(&self) -> u64 {
        self.event_source_healthcheck_polling_interval_ms
    }
}
