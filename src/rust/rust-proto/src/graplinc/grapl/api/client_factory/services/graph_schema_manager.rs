use crate::graplinc::grapl::api::client_factory::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

#[derive(clap::Parser, Debug, Clone)]
pub struct GraphSchemaManagerClientConfig {
    #[clap(long, env)]
    pub graph_schema_manager_client_address: String,
}

impl From<GraphSchemaManagerClientConfig> for GenericGrpcClientConfig {
    fn from(val: GraphSchemaManagerClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.graph_schema_manager_client_address,
        }
    }
}

impl GrpcClientConfig for GraphSchemaManagerClientConfig {}
