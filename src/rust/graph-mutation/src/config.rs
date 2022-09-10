use std::net::SocketAddr;

use rust_proto::graplinc::grapl::api::client_factory::services::{
    GraphSchemaManagerClientConfig,
    UidAllocatorClientConfig,
};

#[derive(clap::Parser, Debug, Clone)]
pub struct GraphDbConfig {
    #[clap(long, env, value_delimiter = ',')]
    /// The address of the graph database
    pub graph_db_addresses: Vec<SocketAddr>,
    #[clap(env)]
    /// The username for the graph database
    pub graph_db_auth_username: String,
    #[clap(env)]
    /// The password for the graph database
    pub graph_db_auth_password: String,
}

#[derive(clap::Parser, Debug, Clone)]
pub struct GraphMutationServiceConfig {
    #[clap(env)]
    /// The address to bind the graph mutation service to
    pub graph_mutation_bind_address: SocketAddr,

    #[clap(flatten)]
    pub uid_allocator_client_config: UidAllocatorClientConfig,

    #[clap(flatten)]
    pub graph_schema_manager_client_config: GraphSchemaManagerClientConfig,

    #[clap(flatten)]
    pub graph_db_config: GraphDbConfig,
}
