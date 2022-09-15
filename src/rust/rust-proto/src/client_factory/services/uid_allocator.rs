use crate::client_factory::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug, Clone)]
pub struct UidAllocatorClientConfig {
    #[clap(long, env)]
    pub uid_allocator_client_address: String,
}

impl From<UidAllocatorClientConfig> for GenericGrpcClientConfig {
    fn from(val: UidAllocatorClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.uid_allocator_client_address,
        }
    }
}

impl GrpcClientConfig for UidAllocatorClientConfig {}
