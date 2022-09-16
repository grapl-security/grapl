pub struct GenericGrpcClientConfig {
    pub address: String,
}

pub trait GrpcClientConfig: clap::Parser + Into<GenericGrpcClientConfig> {}
