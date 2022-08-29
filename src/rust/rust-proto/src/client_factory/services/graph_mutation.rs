use crate::{
    client_factory::grpc_client_config::{
        GenericGrpcClientConfig,
        GrpcClientConfig,
    },
    graplinc::grapl::api::graph_mutation::v1beta1::client::GraphMutationClient,
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

impl GrpcClientConfig for GraphMutationClientConfig {
    type Client = GraphMutationClient;
}
