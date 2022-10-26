use std::net::SocketAddr;

#[derive(clap::Parser, Debug, Clone)]
pub struct GraphQueryProxyConfig {
    /// The tenant id to proxy for
    #[clap(long, env)]
    pub tenant_id: uuid::Uuid,
    #[clap(long, env)]
    /// The address to bind the graph query service to
    pub graph_query_proxy_bind_address: SocketAddr,
}
