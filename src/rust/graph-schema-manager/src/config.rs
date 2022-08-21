use std::net::SocketAddr;

#[derive(clap::Parser, Debug, Clone)]
pub struct SchemaDbConfig {
    #[clap(long, env)]
    schema_db_address: String,
    #[clap(long, env)]
    schema_db_username: String,
    #[clap(long, env)]
    schema_db_password: grapl_config::SecretString,
}

impl grapl_config::ToPostgresUrl for SchemaDbConfig {
    fn to_postgres_url(self) -> grapl_config::PostgresUrl {
        grapl_config::PostgresUrl {
            address: self.schema_db_address,
            username: self.schema_db_username,
            password: self.schema_db_password,
        }
    }
}

#[derive(clap::Parser, Debug, Clone)]
pub struct GraphSchemaManagerConfig {
    #[clap(long, env)]
    /// The address to bind the schema manager service to
    pub graph_schema_manager_bind_address: SocketAddr,

    #[clap(long, env)]
    /// The address to bind the schema manager service to
    pub graph_schema_manager_healthcheck_polling_interval_ms: u64,

    #[clap(flatten)]
    /// Configuration for the Postgres database where we store our tenant-specific schemas
    pub schema_db_config: SchemaDbConfig,
}
