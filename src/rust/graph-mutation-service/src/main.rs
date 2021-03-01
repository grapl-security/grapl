#![allow(warnings)]

// use crate::graph_mutation::graph_mutation_rpc_server::GraphMutationRpc;
// use crate::graph_mutation::{SetEdgeRequest, SetEdgeResponse, SetNodeRequest, SetNodeResponse};
use dgraph_tonic::Status;
use tonic::{Request, Response};

pub mod mutations;
pub mod upsert_manager;

pub use grapl_graph_descriptions::*;
pub use grapl_graph_descriptions::graph_mutation_service::*;
pub use grapl_graph_descriptions::graph_mutation_service::graph_mutation_rpc_server::GraphMutationRpc;

use crate::mutations::node_mutation::NodeUpsertGenerator;

fn main() {
    println!("Hello, world!");
}

#[derive(thiserror::Error, Debug)]
pub enum GraphMutationError {
    #[error("SetNode missing field")]
    SetNodeMissingField{missing_field: &'static str,}
}

impl From<GraphMutationError> for Status {
    fn from(_err: GraphMutationError) -> Status {
        unimplemented!()
    }
}

pub struct GraphMutationService {
}

#[tonic::async_trait]
impl GraphMutationRpc for GraphMutationService {
    async fn set_node(
        &self,
        request: Request<SetNodeRequest>,
    ) -> Result<Response<SetNodeResponse>, Status> {
        let request = request.into_inner();

        let node = match request.node {
            Some(node) => node,
            None => return Err(GraphMutationError::SetNodeMissingField {missing_field: "node"}.into())
        };
        // self.node_upster_generator.
        unimplemented!()
    }

    async fn set_edge(
        &self,
        _request: Request<SetEdgeRequest>,
    ) -> Result<Response<SetEdgeResponse>, Status> {
        unimplemented!()
    }
}
