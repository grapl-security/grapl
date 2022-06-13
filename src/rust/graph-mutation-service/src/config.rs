use std::net::SocketAddr;

use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
pub struct GraphDbConfig {
    #[structopt(env)]
    /// The address of the graph database
    pub graph_db_addresses: Vec<SocketAddr>,
    #[structopt(env)]
    /// The username for the graph database
    pub graph_db_auth_username: String,
    #[structopt(env)]
    /// The password for the graph database
    pub graph_db_auth_password: String,
}

#[derive(StructOpt, Debug, Clone)]
pub struct UidAllocatorClientConfig {
    #[structopt(env)]
    /// The address to connect to for the uid allocator
    pub address: String,
}

#[derive(StructOpt, Debug, Clone)]
pub struct SchemaManagerClientConfig {
    #[structopt(env)]
    /// The address to connect to for the uid allocator
    pub address: String,
}

#[derive(StructOpt, Debug, Clone)]
pub struct GraphMutationServiceConfig {
    #[structopt(env)]
    /// The address to bind the graph mutation service to
    pub graph_mutation_service_bind_address: SocketAddr,

    #[structopt(flatten)]
    pub uid_allocator_client_config: UidAllocatorClientConfig,

    #[structopt(flatten)]
    pub schema_manager_client_config: SchemaManagerClientConfig,

    #[structopt(flatten)]
    pub graph_db_config: GraphDbConfig,
}
