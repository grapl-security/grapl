use rust_proto::protocol::service_client::{
    Connectable,
    NamedService,
};

pub struct GenericGrpcClientConfig {
    pub address: String,
    pub healthcheck_polling_interval_ms: u64,
}

pub trait GrpcClientConfig: clap::Parser + Into<GenericGrpcClientConfig> {
    type Client: NamedService + Connectable;
}
