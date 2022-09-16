use crate::graplinc::grapl::api::client_factory::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug)]
pub struct GraphMutationClientConfig {
    #[clap(long, env)]
    pub graph_mutation_client_address: String,
}

impl From<GraphMutationClientConfig> for GenericGrpcClientConfig {
    fn from(val: GraphMutationClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.graph_mutation_client_address,
        }
    }
}

impl GrpcClientConfig for GraphMutationClientConfig {}
