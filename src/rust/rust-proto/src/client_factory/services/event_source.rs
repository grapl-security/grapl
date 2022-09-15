use crate::client_factory::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug)]
pub struct EventSourceClientConfig {
    #[clap(long, env)]
    pub event_source_client_address: String,
}

impl From<EventSourceClientConfig> for GenericGrpcClientConfig {
    fn from(val: EventSourceClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.event_source_client_address,
        }
    }
}

impl GrpcClientConfig for EventSourceClientConfig {}
