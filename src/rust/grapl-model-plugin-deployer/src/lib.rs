#![warn(clippy::all)]

pub mod server;
pub mod client;

pub mod grapl_model_plugin_deployer {
    tonic::include_proto!("grapl_model_plugin_deployer");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
	todo!("Write some tests!")
    }
}
