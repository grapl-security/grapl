use std::{
    collections::HashMap,
    net::ToSocketAddrs,
};

use generator_sdk::server::{
    self,
    GeneratorServiceConfig,
};
use rust_proto_new::{
    graplinc::grapl::api::{
        graph::v1beta1::GraphDescription,
        plugin_sdk::generators::v1beta1::{
            GeneratedGraph,
            GeneratorApi,
            GeneratorApiError,
            RunGeneratorRequest,
            RunGeneratorResponse,
        },
    },
    protocol::status::Status,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = GeneratorServiceConfig {
        address: "127.0.0.1:5555".to_socket_addrs()?.next().expect("address"),
    };
    let generator = ExampleGenerator {};
    server::exec_service(generator, config).await
}

#[derive(thiserror::Error, Debug)]
pub enum ExampleGeneratorError {
    #[error(transparent)]
    GeneratorApiError(#[from] GeneratorApiError),
}

impl From<ExampleGeneratorError> for Status {
    fn from(e: ExampleGeneratorError) -> Self {
        Status::internal(e.to_string())
    }
}

pub struct ExampleGenerator {}

#[async_trait::async_trait]
impl GeneratorApi for ExampleGenerator {
    type Error = ExampleGeneratorError;

    #[tracing::instrument(skip(self, _data), err)]
    async fn run_generator(
        &self,
        _data: RunGeneratorRequest,
    ) -> Result<RunGeneratorResponse, Self::Error> {
        let nodes = HashMap::new();
        let edges = HashMap::new();
        Ok(RunGeneratorResponse {
            generated_graph: GeneratedGraph {
                graph_description: GraphDescription { nodes, edges },
            },
        })
    }
}
