/*
This module just re-exports the server, client, and protos. No need to modify.
*/

pub mod server;
pub mod client;

pub mod model_plugin_deployer {
    // In the future, this will be in rust-proto.
    tonic::include_proto!("graplinc.grapl.api.model_plugin_deployer.v1");
    // Everything in that package is now availabe in this namespace, e.g.
    // use crate::model_plugin_deployer::SchemaType

    pub fn get_socket_addr() -> String {
        let env_var_name = "MODEL_PLUGIN_DEPLOYER_GRPC_PORT";
        let port = std::env::var(env_var_name).expect(env_var_name);
        return format!("[::1]:{}", port);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        todo!("Write some tests!")
    }
}
