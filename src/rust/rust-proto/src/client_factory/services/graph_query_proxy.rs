use crate::client_factory::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug, Clone)]
pub struct GraphQueryProxyClientConfig {
    #[clap(long, env)]
    pub graph_query_proxy_client_address: String,
}

impl From<GraphQueryProxyClientConfig> for GenericGrpcClientConfig {
    fn from(val: GraphQueryProxyClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.graph_query_proxy_client_address,
        }
    }
}

impl GrpcClientConfig for GraphQueryProxyClientConfig {}
