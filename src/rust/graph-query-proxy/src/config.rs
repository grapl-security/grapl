use std::net::SocketAddr;

use rust_proto::protocol::endpoint::Endpoint;

#[derive(clap::Parser, Debug, Clone)]
pub struct GraphQueryProxyConfig {
    /// The tenant id to proxy for
    #[clap(env)]
    pub tenant_id: uuid::Uuid,
    #[clap(env)]
    /// The address to bind the graph query service to
    pub graph_query_proxy_bind_address: SocketAddr,
    #[clap(long, env, value_delimiter = ',')]
    /// The address of the graph query service
    pub graph_query_service_client_urls: Vec<Endpoint>,
}
