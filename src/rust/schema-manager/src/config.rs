use std::net::SocketAddr;

#[derive(clap::Parser, Debug, Clone)]
pub struct SchemaDbConfig {
    #[clap(env)]
    /// The hostname of the counter database
    schema_db_hostname: String,

    #[clap(env)]
    /// The username to use when connecting to the counter database
    schema_db_username: String,

    #[clap(env)]
    /// The password to use when connecting to the counter database
    schema_db_password: String,

    #[clap(env)]
    /// The port to use when connecting to the counter database
    schema_db_port: u16,
}

impl SchemaDbConfig {
    /// Returns the postgres connection url
    pub fn to_postgres_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.schema_db_username,
            self.schema_db_password,
            self.schema_db_hostname,
            self.schema_db_port,
        )
    }
}

#[derive(clap::Parser, Debug, Clone)]
pub struct SchemaServiceConfig {
    #[clap(env)]
    /// The address to bind the schema manager service to
    pub schema_service_bind_address: SocketAddr,

    #[clap(flatten)]
    /// Configuration for the Postgres database where we store our tenant-specific schemas
    pub schema_db_config: SchemaDbConfig,
}
