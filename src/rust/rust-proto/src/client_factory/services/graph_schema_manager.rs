use crate::{
    client_factory::grpc_client_config::{
        GenericGrpcClientConfig,
        GrpcClientConfig,
    },
    graplinc::grapl::api::graph_schema_manager::v1beta1::client::GraphSchemaManagerClient,
};

#[derive(clap::Parser, Debug)]
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

impl GrpcClientConfig for GraphSchemaManagerClientConfig {
    type Client = GraphSchemaManagerClient;
}
