use std::net::SocketAddr;

#[derive(clap::Parser, Clone, Debug)]
#[clap(name = "graph-generator", about = "Graph Generator Service")]
pub struct GraphDbConfig {
    #[clap(long, env)]
    /// The address of the graph database
    pub graph_db_addresses: Vec<SocketAddr>,
    #[clap(env)]
    /// The username for the graph database
    pub graph_db_auth_username: String,
    #[clap(env)]
    /// The password for the graph database
    pub graph_db_auth_password: secrecy::SecretString,
}

#[derive(clap::Parser, Debug, Clone)]
pub struct GraphQueryServiceConfig {
    #[clap(env)]
    /// The address to bind the graph query service to
    pub graph_query_service_bind_address: SocketAddr,

    #[clap(flatten)]
    pub graph_db_config: GraphDbConfig,
}
