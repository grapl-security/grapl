use crate::graplinc::grapl::api::{
    client_factory::grpc_client_config::{
        GenericGrpcClientConfig,
        GrpcClientConfig,
    },
    graph_query_service::v1beta1::client::GraphQueryClient,
};

#[derive(clap::Parser, Debug)]
pub struct GraphQueryClientConfig {
    #[clap(long, env)]
    pub graph_query_client_address: String,
}

impl From<GraphQueryClientConfig> for GenericGrpcClientConfig {
    fn from(val: GraphQueryClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.graph_query_client_address,
        }
    }
}

impl GrpcClientConfig for GraphQueryClientConfig {
    type Client = GraphQueryClient;
}
