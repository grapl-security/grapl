use std::net::SocketAddr;

use rust_proto::protocol::service_client::{
    Connectable,
    NamedService,
};

pub trait GrpcClientConfig {
    type Client: NamedService + Connectable;

    fn address(&self) -> SocketAddr;
    fn healthcheck_polling_interval_ms(&self) -> u64;
}
