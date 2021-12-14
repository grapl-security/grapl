use std::net::SocketAddr;
use structopt::StructOpt;

pub mod client;
pub mod server;

#[derive(StructOpt, Debug)]
pub struct PluginRegistryServiceConfig {
    #[structopt(env)]
    plugin_s3_bucket_aws_account_id: String,
    #[structopt(env)]
    plugin_s3_bucket_name: String,
    #[structopt(env)]
    grpc_address: SocketAddr,
    #[structopt(env)]
    plugin_registry_psql_address: String,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
