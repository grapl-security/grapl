use std::net::SocketAddr;

pub mod server;

#[derive(clap::Parser, Debug)]
pub struct PluginBootstrapServiceConfig {
    #[clap(long, env)]
    pub plugin_registry_bind_address: SocketAddr,
    #[clap(long, env)]
    pub plugin_binary_path: std::path::PathBuf,
    #[clap(long, env)]
    pub plugin_certificate_path: std::path::PathBuf,
    #[clap(long, env)]
    pub plugin_registry_polling_interval_ms: u64,
}
