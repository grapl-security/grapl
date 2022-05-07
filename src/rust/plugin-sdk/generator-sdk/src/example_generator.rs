use std::{collections::HashMap, net::{ToSocketAddrs}};

use generator_sdk::server::{self, GeneratorServiceConfig};
use rust_proto_new::{graplinc::grapl::api::{plugin_sdk::generators::v1beta1::{
    GeneratorApi,
    GeneratorApiError, RunGeneratorResponse, RunGeneratorRequest, GeneratedGraph,
}, graph::v1beta1::GraphDescription}, protocol::status::Status};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = GeneratorServiceConfig {
        address: "127.0.0.1:5555".to_socket_addrs()?.into_iter().next().expect("address"),
    };
    let generator = ExampleGenerator {};
    server::exec_service(generator, config).await
}

#[thiserror::Error]
pub enum ExampleGeneratorError {
    #[error(transparent)]
    GeneratorApiError(#[from] GeneratorApiError)
}

impl From<GeneratorApiError> for Status {
    fn from(e: IngressApiError) -> Self {
        Status::internal(e.to_string())
    }
}

pub struct ExampleGenerator {}

impl GeneratorApi for ExampleGenerator {
    type Error = GeneratorApiError;

    #[tracing::instrument(skip(self, _data), err)]
    fn run_generator(
        &self,
        _data: RunGeneratorRequest
    ) -> Result<RunGeneratorResponse, Self::Error> {
        let nodes = HashMap::new();
        let edges = HashMap::new();
        Ok(RunGeneratorResponse {
            generated_graph: GeneratedGraph {
                graph_description: GraphDescription { nodes, edges }
            }
        })
    }
}
