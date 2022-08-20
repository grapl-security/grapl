use std::net::SocketAddr;

use secrecy::ExposeSecret;

#[derive(clap::Parser, Clone, Debug)]
pub struct GraphDbConfig {
    #[clap(long, env)]
    /// The address of the graph database
    pub graph_db_addresses: Vec<SocketAddr>,
    #[clap(long, env)]
    /// The username for the graph database
    pub graph_db_auth_username: String,
    #[clap(long, env)]
    /// The password for the graph database
    pub graph_db_auth_password: secrecy::SecretString,
}

impl GraphDbConfig {
    pub async fn connect(&self) -> Result<scylla::Session, Box<dyn std::error::Error>> {
        let mut scylla_config = scylla::SessionConfig::new();
        scylla_config.add_known_nodes_addr(&self.graph_db_addresses[..]);
        scylla_config.auth_username = Some(self.graph_db_auth_username.to_owned());
        scylla_config.auth_password = Some(self.graph_db_auth_password.expose_secret().to_owned());

        Ok(scylla::Session::connect(scylla_config).await?)
    }
}

#[derive(clap::Parser, Debug, Clone)]
pub struct DbSchemaManagerServiceConfig {
    #[clap(env)]
    /// The address to bind the graph query service to
    pub db_schema_manager_bind_address: SocketAddr,

    #[clap(flatten)]
    pub graph_db_config: GraphDbConfig,
}
