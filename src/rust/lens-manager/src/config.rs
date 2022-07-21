use std::net::SocketAddr;

#[derive(clap::Parser, Clone, Debug)]
#[clap(name = "lens-creator", about = "Lens Creator Service")]
pub struct GraphDbConfig {
    #[clap(long, env)]
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
pub struct LensManagerServiceConfig {
    #[clap(env)]
    /// The address to bind the lens manager service to
    pub lens_manager_service_bind_address: SocketAddr,

    #[clap(flatten)]
    pub graph_db_config: GraphDbConfig,
}
