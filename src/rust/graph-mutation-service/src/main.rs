#![allow(warnings)]

use dgraph_tonic::{Status, Client as DgraphClient};
use tonic::{Request, Response};

pub mod mutations;
pub mod upsert_manager;

pub use grapl_graph_descriptions::*;
pub use grapl_graph_descriptions::graph_mutation_service::*;
pub use grapl_graph_descriptions::graph_mutation_service::graph_mutation_rpc_server::GraphMutationRpc;

use crate::mutations::node_mutation::NodeUpsertGenerator;
use crate::upsert_manager::UpsertManager;
use graph_mutation_service_lib::reverse_resolver::{ReverseEdgeResolver, ReverseEdgeResolverError};

fn main() {
    println!("Hello, world!");
}

#[derive(thiserror::Error, Debug)]
pub enum GraphMutationError {
    #[error("SetNode missing field")]
    SetNodeMissingField { missing_field: &'static str },
    #[error("SetEdge missing field")]
    SetEdgeMissingField { missing_field: &'static str },
    #[error("Unexpected number of reverse edges")]
    UnexpectedNumberOfReverseEdges { forward_edge: String, reverse_edge_count: usize },
}

impl From<GraphMutationError> for Status {
    fn from(_err: GraphMutationError) -> Status {
        unimplemented!()
    }
}


impl From<ReverseEdgeResolverError> for Status {
    fn from(_err: ReverseEdgeResolverError) -> Status {
        unimplemented!()
    }
}


pub struct GraphMutationService {
    reverse_edge_resolver: ReverseEdgeResolver,
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
            None => return Err(GraphMutationError::SetNodeMissingField { missing_field: "node" }.into())
        };
        let dgraph_client = std::sync::Arc::new(DgraphClient::new("http://127.0.0.1:9080").expect("Failed to create dgraph client."));
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
        };
        let uid = upsert_manager.upsert_node(&node).await;
        unimplemented!()
    }

    async fn set_edge(
        &self,
        request: Request<SetEdgeRequest>,
    ) -> Result<Response<SetEdgeResponse>, Status> {
        let request = request.into_inner();

        let edge = match request.edge {
            Some(edge) => edge,
            None => return Err(GraphMutationError::SetEdgeMissingField { missing_field: "edge" }.into())
        };
        let mut reversed = self.reverse_edge_resolver.resolve_reverse_edges(vec![edge.clone()]).await?;
        if reversed.len() > 1 {
            return Err(
                GraphMutationError::UnexpectedNumberOfReverseEdges {
                    forward_edge: edge.edge_name.to_owned(),
                    reverse_edge_count: reversed.len() }.into(),
            );
        }
        let reversed = reversed.remove(0);
        let mut upsert_manager = UpsertManager {
            dgraph_client: dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
        };
        let uid = upsert_manager.upsert_edge(edge, reversed).await;
        unimplemented!()
    }
}
