use std::net::SocketAddr;

use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
pub struct SchemaDbConfig {
    #[structopt(env)]
    /// The hostname of the counter database
    schema_db_hostname: String,

    #[structopt(env)]
    /// The username to use when connecting to the counter database
    schema_db_username: String,

    #[structopt(env)]
    /// The password to use when connecting to the counter database
    schema_db_password: String,

    #[structopt(env)]
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

#[derive(StructOpt, Debug, Clone)]
pub struct SchemaServiceConfig {
    #[structopt(env)]
    /// The address to bind the schema manager service to
    pub schema_servicebind_address: SocketAddr,

    #[structopt(flatten)]
    /// Configuration for the Postgres database where we store our tenant-specific schemas
    pub schema_db_config: SchemaDbConfig,
}
