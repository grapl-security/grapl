use std::net::SocketAddr;

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
    pub graph_db_config: GraphDbConfig,
}
