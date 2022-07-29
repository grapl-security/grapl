use crate::protocol::service_client::Connectable;

pub struct GenericGrpcClientConfig {
    pub address: String,
}

pub trait GrpcClientConfig: clap::Parser + Into<GenericGrpcClientConfig> {
    type Client: Connectable;
}
