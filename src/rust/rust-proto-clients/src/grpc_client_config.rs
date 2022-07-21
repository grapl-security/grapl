use rust_proto::protocol::service_client::{
    Connectable,
    NamedService,
};

pub struct GenericGrpcClientConfig {
    pub address: String,
}

pub trait GrpcClientConfig: clap::Parser + Into<GenericGrpcClientConfig> {
    type Client: NamedService + Connectable;
}
