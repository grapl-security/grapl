use crate::{
    client_factory::grpc_client_config::{
        GenericGrpcClientConfig,
        GrpcClientConfig,
    },
    graplinc::grapl::api::schema_manager::v1beta1::client::SchemaManagerClient,
};

#[derive(clap::Parser, Debug)]
pub struct SchemaManagerClientConfig {
    #[clap(long, env)]
    pub schema_manager_client_address: String,
}

impl From<SchemaManagerClientConfig> for GenericGrpcClientConfig {
    fn from(val: SchemaManagerClientConfig) -> Self {
        GenericGrpcClientConfig {
            address: val.schema_manager_client_address,
        }
    }
}

impl GrpcClientConfig for SchemaManagerClientConfig {
    type Client = SchemaManagerClient;
}
