use std::net::SocketAddr;

use structopt::StructOpt;

pub mod client;
pub mod server;

#[derive(StructOpt, Debug)]
pub struct PluginBootstrapServiceConfig {
    #[structopt(env)]
    pub plugin_registry_bind_address: SocketAddr,
    #[structopt(env)]
    pub plugin_binary_path: std::path::PathBuf,
    #[structopt(env)]
    pub plugin_certificate_path: std::path::PathBuf,
}



