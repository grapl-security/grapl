// #![warn(clippy::all)]
#![allow(warnings)]

pub mod client;
pub mod grapl_model_plugin_deployer_request;
pub mod grapl_request_meta;
pub mod server;

pub mod grapl_model_plugin_deployer_proto {
    tonic::include_proto!("grapl_model_plugin_deployer");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        todo!("Write some tests!")
    }
}
