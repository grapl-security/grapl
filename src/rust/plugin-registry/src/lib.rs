use std::net::SocketAddr;

use structopt::StructOpt;

pub mod client;
mod db_client;
mod deploy_plugin;
pub mod error;
pub mod nomad_cli;
pub mod nomad_client;
pub mod server;
pub mod static_files;

#[derive(StructOpt, Debug)]
pub struct PluginRegistryServiceConfig {
    #[structopt(env)]
    plugin_s3_bucket_aws_account_id: String,
    #[structopt(env)]
    plugin_s3_bucket_name: String,
    #[structopt(env)]
    plugin_registry_bind_address: SocketAddr,
    #[structopt(env)]
    plugin_registry_db_hostname: String,
    #[structopt(env)]
    plugin_registry_db_port: u16,
    #[structopt(env)]
    plugin_registry_db_username: String,
    #[structopt(env)]
    plugin_registry_db_password: String,
}
