/// This Generator shows a very basic way to build a Generator, and will also
/// help inform Grapl Engineers of ways to simplify this API before we ship to
/// customers.
use std::collections::HashMap;

use generator_sdk::server::{
    self,
    GeneratorServiceConfig,
};
use rust_proto_new::{
    graplinc::grapl::api::{
        graph::v1beta1::GraphDescription,
        plugin_sdk::generators::v1beta1::{
            server::GeneratorApi,
            GeneratedGraph,
            RunGeneratorRequest,
            RunGeneratorResponse,
        },
    },
    protocol::status::Status,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = GeneratorServiceConfig::from_env_vars();
    let generator = ExampleGenerator {};
    server::exec_service(generator, config).await
}

/// An example, silly error class
#[derive(thiserror::Error, Debug)]
pub enum ExampleGeneratorError {
    #[error("DataStartsWithHELLO")]
    DataStartsWithHELLO,
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

    #[tracing::instrument(skip(self, request), err)]
    async fn run_generator(
        &self,
        request: RunGeneratorRequest,
    ) -> Result<RunGeneratorResponse, Self::Error> {
        if "HELLO".as_bytes() == &request.data[0..5] {
            return Err(ExampleGeneratorError::DataStartsWithHELLO);
        }
        let nodes = HashMap::new();
        let edges = HashMap::new();
        Ok(RunGeneratorResponse {
            generated_graph: GeneratedGraph {
                graph_description: GraphDescription { nodes, edges },
            },
        })
    }
}