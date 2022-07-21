use rust_proto::protocol::service_client::{
    Connectable,
    NamedService,
};

pub trait GrpcClientConfig: clap::Parser {
    type Client: NamedService + Connectable;

    fn address(&self) -> &str;
    fn healthcheck_polling_interval_ms(&self) -> u64;
}

/*
pub trait ParseFromEnv
where
    Self: Sized,
{
    fn parse_from_env() -> Self;
}

/// A blanket implementation that lets users consume from env without an
/// explicit Cargo.toml usage of Clap
impl<C> ParseFromEnv for C
where
    C: GrpcClientConfig,
{
    fn parse_from_env() -> Self {
        Self::parse()
    }
}
 */
