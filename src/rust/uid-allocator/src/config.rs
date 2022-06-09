use std::net::SocketAddr;

use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
pub struct CounterDbConfig {
    #[structopt(env)]
    /// The hostname of the counter database
    counter_db_hostname: String,

    #[structopt(env)]
    /// The username to use when connecting to the counter database
    counter_db_username: String,

    #[structopt(env)]
    /// The password to use when connecting to the counter database
    counter_db_password: String,

    #[structopt(env)]
    /// The port to use when connecting to the counter database
    counter_db_port: u16,
}

impl CounterDbConfig {
    /// Returns the postgres connection url
    pub fn to_postgres_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.counter_db_username,
            self.counter_db_password,
            self.counter_db_hostname,
            self.counter_db_port,
        )
    }
}

#[derive(StructOpt, Debug, Clone)]
pub struct UidAllocatorServiceConfig {
    #[structopt(env)]
    /// The address to bind the uid allocator service to
    pub uid_allocator_bind_address: SocketAddr,

    #[structopt(env)]
    /// Default allocation size indicates how many uids to allocate for a tenant if the
    /// client requests an allocation of size `0`.
    /// Consider values of 10, 100, or 1_000
    /// Should not be a value greater than `maximum_allocation_size` and must not be `0`.
    pub default_allocation_size: u32,

    #[structopt(env)]
    /// How many uids to preallocate when our last preallocation is exhausted
    /// While this can be as large as a u32, it is not recommended to set this to a value
    /// too high. Consider values such as 100, 1000, or 10_000 instead.
    pub preallocation_size: u32,

    #[structopt(env)]
    /// The maximum size of an allocation that we'll hand out to a given client for a
    /// request. Similar to the `preallocation_size` field, this is a value that can be
    /// a full 32bit integer, but is not recommended to be too large. It should also
    /// always me smaller than the preallocation_size.
    /// Consider values such as 10, 100, or 1_000.
    pub maximum_allocation_size: u32,

    #[structopt(flatten)]
    /// Configuration for the Postgres database where we store our tenant-specific counters
    pub counter_db_config: CounterDbConfig,
}

impl UidAllocatorServiceConfig {
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.default_allocation_size == 0 {
            return Err("default_allocation_size must be greater than 0".into());
        }

        if self.preallocation_size == 0 {
            return Err("preallocation_size must be greater than 0".into());
        }

        if self.maximum_allocation_size == 0 {
            return Err("maximum_allocation_size must be greater than 0".into());
        }

        if self.preallocation_size < self.maximum_allocation_size {
            return Err("preallocation_size must be greater than maximum_allocation_size".into());
        }

        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
pub struct UidAllocatorClientConfig {
    #[structopt(env)]
    /// The address to connect the uid allocator client to
    pub uid_allocator_connect_address: SocketAddr,

    #[structopt(env)]
    /// The size for the client to request when allocating uids
    pub allocation_size: u32,
}
