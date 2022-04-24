use std::net::SocketAddr;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
pub struct CounterDbConfig {
    #[structopt(env)]
    /// The address of the counter database
    counter_postgres_connect_address: SocketAddr,

    #[structopt(env)]
    /// The username to use when connecting to the counter database
    counter_postgres_username: String,

    #[structopt(env)]
    /// The password to use when connecting to the counter database
    counter_postgres_password: String,
}

impl CounterDbConfig {
    /// Returns the postgres connection url
    pub fn to_postgres_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}",
            self.counter_postgres_username,
            self.counter_postgres_password,
            self.counter_postgres_connect_address
        )
    }
}

#[derive(StructOpt, Debug, Clone)]
pub struct UidAllocatorServiceConfig {
    #[structopt(env)]
    /// The address to bind the uid allocator service to
    pub uid_allocator_bind_address: SocketAddr,

    #[structopt(env)]
    /// The number of uids to pre-allocate in order to avoid the need to
    /// hit the database too frequently.
    pub preallocation_size: u32,

    #[structopt(env)]
    /// The maximum allocation range we'll return to a client.
    pub maximum_allocation_size: u32,

    #[structopt(flatten)]
    /// Configuration for the Postgres database where we store our tenant-specific counters
    pub counter_db_config: CounterDbConfig,
}